//! Thin HTTP wrapper over the Chroma SDK REST API (spec §4).
//!
//! All calls time out after 2 seconds unless overridden by
//! `RAZER_CHROMA_TIMEOUT_MS` at construction time.
//! The base URL defaults to `http://localhost:54235` but can be overridden via
//! `RAZER_CHROMA_BASE_URL` for tests.

use crate::error::{ChromaError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE: &str = "http://localhost:54235/razer/chromasdk";

fn classify_send_error(e: reqwest::Error) -> ChromaError {
    if e.is_connect() || e.is_timeout() {
        ChromaError::SynapseNotRunning
    } else {
        ChromaError::Network(e)
    }
}

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
        let base =
            std::env::var("RAZER_CHROMA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string());
        Self::with_base_url(&base)
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
        let resp = self
            .http
            .post(&self.base_url)
            .json(&payload)
            .send()
            .await
            .map_err(classify_send_error)?;
        let status = resp.status();
        if status.is_success() {
            return resp
                .json::<Session>()
                .await
                .map_err(|e| ChromaError::SessionInitFailed(e.to_string()));
        }
        // Disambiguate: if Synapse's web root is reachable but the Chroma
        // endpoint returns 404/503, the Chroma module itself is disabled.
        if status.as_u16() == 404 || status.as_u16() == 503 {
            let root = self.base_url.trim_end_matches("/razer/chromasdk");
            if self
                .http
                .get(root)
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false)
            {
                return Err(ChromaError::ChromaModuleDisabled);
            }
        }
        Err(ChromaError::SessionInitFailed(format!("HTTP {status}")))
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
            return Err(ChromaError::EffectPutFailed(format!(
                "PUT {} -> HTTP {}",
                url,
                resp.status()
            )));
        }
        Ok(())
    }

    /// PUT a heartbeat to `<uri>/heartbeat`.
    ///
    /// Synapse's Chroma REST endpoint rejects bodyless PUTs with `HTTP 411
    /// Length Required`, so we send an explicit empty JSON object. This keeps
    /// the heartbeat cycle from snowballing into `Reconnecting` → `Degraded`
    /// on long-lived sessions (observed on live hardware, T3.7).
    pub async fn heartbeat(&self, session_uri: &str) -> Result<()> {
        let url = format!("{}/heartbeat", session_uri.trim_end_matches('/'));
        let resp = self
            .http
            .put(&url)
            .json(&serde_json::json!({}))
            .send()
            .await?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ChromaError;

    #[tokio::test]
    async fn connection_refused_maps_to_synapse_not_running() {
        // Port 1 is reserved and will refuse connection on most systems.
        let client = ChromaClient::with_base_url("http://127.0.0.1:1");
        let err = client
            .init_session(serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                ChromaError::SynapseNotRunning | ChromaError::Network(_)
            ),
            "got: {err:?}"
        );
    }

    #[tokio::test]
    async fn chroma_module_disabled_is_distinct_from_synapse_not_running() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        // Root probe succeeds → Synapse is up.
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;
        // Chroma endpoint returns 404 → module off.
        Mock::given(method("POST"))
            .and(path("/razer/chromasdk"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let client = ChromaClient::with_base_url(&server.uri());
        let err = client
            .init_session(serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(
            matches!(err, ChromaError::ChromaModuleDisabled),
            "got {err:?}"
        );
    }
}
