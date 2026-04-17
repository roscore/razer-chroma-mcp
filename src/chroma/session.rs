//! Owns the Chroma session lifecycle: init, heartbeat loop, reconnect, graceful drop.
//!
//! Per spec §3: heartbeat 5s interval, 2s HTTP timeout, 3 consecutive failures triggers reconnect.
//! Tests can override all three via `SessionConfig`.

use crate::chroma::client::{Author, ChromaClient, InitPayload};
use crate::chroma::effects::EffectEnvelope;
use crate::error::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub heartbeat_interval: Duration,
    pub reconnect_threshold: u32,
    pub reinit_max: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(5),
            reconnect_threshold: 3,
            reinit_max: 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Uninitialized,
    Healthy,
    Reconnecting { failures: u32 },
    Degraded,
    ShutDown,
}

#[derive(Debug)]
struct Inner {
    uri: String,
    status: SessionStatus,
    failures: u32,
}

#[derive(Clone)]
pub struct Session {
    client: ChromaClient,
    cfg: SessionConfig,
    inner: Arc<RwLock<Inner>>,
    cancel: CancellationToken,
    hb_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

pub fn default_init_payload() -> InitPayload {
    InitPayload {
        title: "razer-chroma-mcp".into(),
        description: "MCP server for Razer Chroma".into(),
        author: Author {
            name: "roscore".into(),
            contact: "https://github.com/roscore".into(),
        },
        device_supported: vec![
            "keyboard".into(),
            "mouse".into(),
            "mousepad".into(),
            "headset".into(),
            "keypad".into(),
            "chromalink".into(),
        ],
        category: "application".into(),
    }
}

impl Session {
    pub async fn init(client: ChromaClient, cfg: SessionConfig) -> Result<Self> {
        let session = client.init_session(default_init_payload()).await?;
        let inner = Arc::new(RwLock::new(Inner {
            uri: session.uri.clone(),
            status: SessionStatus::Healthy,
            failures: 0,
        }));
        let cancel = CancellationToken::new();
        let me = Self {
            client,
            cfg,
            inner,
            cancel: cancel.clone(),
            hb_handle: Arc::new(tokio::sync::Mutex::new(None)),
        };
        let handle = tokio::spawn(me.clone().run_heartbeat(cancel));
        *me.hb_handle.lock().await = Some(handle);
        Ok(me)
    }

    pub async fn status(&self) -> SessionStatus {
        self.inner.read().await.status
    }

    pub async fn uri(&self) -> String {
        self.inner.read().await.uri.clone()
    }

    pub async fn put_effect(&self, device_path: &str, effect: &EffectEnvelope) -> Result<()> {
        let uri = self.uri().await;
        self.client.put_effect(&uri, device_path, effect).await
    }

    async fn run_heartbeat(self, cancel: CancellationToken) {
        let mut ticker = tokio::time::interval(self.cfg.heartbeat_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = ticker.tick() => {
                    let uri = self.uri().await;
                    match self.client.heartbeat(&uri).await {
                        Ok(()) => {
                            let mut g = self.inner.write().await;
                            g.failures = 0;
                            if matches!(g.status, SessionStatus::Reconnecting { .. }) {
                                g.status = SessionStatus::Healthy;
                            }
                        }
                        Err(e) => {
                            let should_reconnect;
                            {
                                let mut g = self.inner.write().await;
                                g.failures += 1;
                                g.status = SessionStatus::Reconnecting { failures: g.failures };
                                should_reconnect = g.failures >= self.cfg.reconnect_threshold;
                                tracing::warn!(failures = g.failures, "heartbeat failure: {e}");
                            }
                            if should_reconnect {
                                self.reconnect().await;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Linear-backoff reconnect. Each `init_session` call is bounded by the
    /// ChromaClient HTTP timeout (2s by default, see `RAZER_CHROMA_TIMEOUT_MS`).
    /// After `reinit_max` failed attempts, status transitions to `Degraded`.
    async fn reconnect(&self) {
        for attempt in 1..=self.cfg.reinit_max {
            match self.client.init_session(default_init_payload()).await {
                Ok(s) => {
                    let mut g = self.inner.write().await;
                    g.uri = s.uri;
                    g.status = SessionStatus::Healthy;
                    g.failures = 0;
                    tracing::info!(attempt, "session reconnected");
                    return;
                }
                Err(e) => {
                    tracing::warn!(attempt, "reconnect attempt failed: {e}");
                    tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
            }
        }
        self.inner.write().await.status = SessionStatus::Degraded;
        tracing::error!("session marked DEGRADED after {} reinit attempts", self.cfg.reinit_max);
    }

    pub async fn shutdown(&self) {
        self.cancel.cancel();
        if let Some(h) = self.hb_handle.lock().await.take() {
            let _ = h.await;
        }
        let uri = self.uri().await;
        // Best-effort NONE + delete
        let _ = self.client.put_effect(&uri, "keyboard", &EffectEnvelope::none()).await;
        let _ = self.client.delete_session(&uri).await;
        self.inner.write().await.status = SessionStatus::ShutDown;
    }
}
