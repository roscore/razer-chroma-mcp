//! Thin HTTP wrapper over the Chroma SDK REST API (spec §4).
//!
//! All calls time out after 2 seconds unless overridden by
//! `RAZER_CHROMA_TIMEOUT_MS` at construction time.

use crate::error::{ChromaError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE: &str = "http://localhost:54235/razer/chromasdk";

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    #[serde(rename = "sessionid")]
    pub session_id: u64,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitPayload {
    pub title: String,
    pub description: String,
    pub author: Author,
    pub device_supported: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Author {
    pub name: String,
    pub contact: String,
}

#[derive(Debug, Clone)]
pub struct ChromaClient {
    http: Client,
    base_url: String,
}

impl Default for ChromaClient {
    fn default() -> Self {
        Self::with_base_url(DEFAULT_BASE)
    }
}

impl ChromaClient {
    pub fn with_base_url(base: &str) -> Self {
        let timeout_ms = std::env::var("RAZER_CHROMA_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2_000);
        let http = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("reqwest client build");
        let base_url = if base.ends_with("/razer/chromasdk") {
            base.to_string()
        } else {
            format!("{}/razer/chromasdk", base.trim_end_matches('/'))
        };
        Self { http, base_url }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn init_session<T: Serialize>(&self, payload: T) -> Result<Session> {
        let resp = self.http.post(&self.base_url).json(&payload).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::SessionInitFailed(format!("HTTP {}", resp.status())));
        }
        resp.json::<Session>()
            .await
            .map_err(|e| ChromaError::SessionInitFailed(e.to_string()))
    }

    /// PUT an effect to the per-session endpoint `<uri>/<device>`.
    pub async fn put_effect(
        &self,
        session_uri: &str,
        device_path: &str,
        effect: &crate::chroma::effects::EffectEnvelope,
    ) -> Result<()> {
        let url = format!("{}/{}", session_uri.trim_end_matches('/'), device_path);
        let resp = self.http.put(&url).json(effect).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::SessionInitFailed(format!(
                "PUT {} -> HTTP {}",
                url,
                resp.status()
            )));
        }
        Ok(())
    }

    /// PUT a heartbeat to `<uri>/heartbeat`.
    pub async fn heartbeat(&self, session_uri: &str) -> Result<()> {
        let url = format!("{}/heartbeat", session_uri.trim_end_matches('/'));
        let resp = self.http.put(&url).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::HeartbeatFailed {
                failures: 1,
                cause: format!("HTTP {}", resp.status()),
            });
        }
        Ok(())
    }

    /// DELETE the session to tear it down cleanly.
    pub async fn delete_session(&self, session_uri: &str) -> Result<()> {
        let resp = self.http.delete(session_uri).send().await?;
        if !resp.status().is_success() {
            tracing::warn!(status = %resp.status(), "delete_session returned non-2xx; ignoring");
        }
        Ok(())
    }
}
