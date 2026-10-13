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

/// Env var overriding the per-request HTTP timeout in milliseconds.
pub const TIMEOUT_ENV: &str = "RAZER_CHROMA_TIMEOUT_MS";
/// Default per-request HTTP timeout (spec §3).
pub const DEFAULT_TIMEOUT_MS: u64 = 2_000;
/// Largest accepted override. Larger values would stall the 5s heartbeat loop.
pub const MAX_TIMEOUT_MS: u64 = 60_000;

/// Resolve the timeout from an optional raw env value.
///
/// Returns the timeout to use plus a warning message when `raw` was present
/// but invalid (empty, non-integer, negative, zero, or above
/// [`MAX_TIMEOUT_MS`]). Invalid values fall back to [`DEFAULT_TIMEOUT_MS`].
pub fn resolve_timeout_ms(raw: Option<&str>) -> (u64, Option<String>) {
    let Some(raw) = raw else {
        return (DEFAULT_TIMEOUT_MS, None);
    };
    let reason = match raw.trim().parse::<u64>() {
        Ok(0) => "must be greater than 0",
        Ok(ms) if ms > MAX_TIMEOUT_MS => "exceeds maximum",
        Ok(ms) => return (ms, None),
        Err(_) => "not a positive integer",
    };
    let warning = format!(
        "{TIMEOUT_ENV}={raw:?} is invalid ({reason}; allowed 1..={MAX_TIMEOUT_MS}); \
         using default {DEFAULT_TIMEOUT_MS}ms"
    );
    (DEFAULT_TIMEOUT_MS, Some(warning))
}

fn timeout_from_env() -> u64 {
    let raw = std::env::var(TIMEOUT_ENV).ok();
    let (ms, warning) = resolve_timeout_ms(raw.as_deref());
    if let Some(w) = warning {
        tracing::warn!("{w}");
    }
    ms
}

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
        let timeout_ms = timeout_from_env();
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

    #[test]
    fn timeout_unset_uses_default_without_warning() {
        assert_eq!(resolve_timeout_ms(None), (DEFAULT_TIMEOUT_MS, None));
    }

    #[test]
    fn timeout_valid_value_is_used() {
        assert_eq!(resolve_timeout_ms(Some("1500")), (1500, None));
        assert_eq!(resolve_timeout_ms(Some(" 750 ")), (750, None));
    }

    #[test]
    fn timeout_bounds_are_inclusive() {
        assert_eq!(resolve_timeout_ms(Some("1")).0, 1);
        let max = MAX_TIMEOUT_MS.to_string();
        assert_eq!(resolve_timeout_ms(Some(&max)), (MAX_TIMEOUT_MS, None));
    }

    fn assert_falls_back(raw: &str) {
        let (ms, warn) = resolve_timeout_ms(Some(raw));
        assert_eq!(ms, DEFAULT_TIMEOUT_MS, "raw={raw:?}");
        let warn = warn.unwrap_or_else(|| panic!("expected warning for {raw:?}"));
        assert!(
            warn.contains(TIMEOUT_ENV),
            "warning should name env var: {warn}"
        );
    }

    #[test]
    fn timeout_zero_falls_back_with_warning() {
        assert_falls_back("0");
    }

    #[test]
    fn timeout_negative_falls_back_with_warning() {
        assert_falls_back("-5");
    }

    #[test]
    fn timeout_non_numeric_falls_back_with_warning() {
        assert_falls_back("abc");
        assert_falls_back("2s");
        assert_falls_back("1.5");
        assert_falls_back("");
    }

    #[test]
    fn timeout_above_max_falls_back_with_warning() {
        assert_falls_back(&(MAX_TIMEOUT_MS + 1).to_string());
        assert_falls_back("99999999999999999999999");
    }
}
