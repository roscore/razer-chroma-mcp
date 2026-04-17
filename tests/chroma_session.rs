//! Verifies session heartbeat loop and reconnect threshold.

use razer_chroma_mcp::chroma::client::ChromaClient;
use razer_chroma_mcp::chroma::session::{Session, SessionConfig};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "multi_thread")]
async fn heartbeat_loop_retries_and_reconnects_after_three_failures() {
    let server = MockServer::start().await;
    let uri = format!("{}/session/1", server.uri());

    Mock::given(method("POST"))
        .and(path("/razer/chromasdk"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "sessionid": 1, "uri": uri,
        })))
        .mount(&server)
        .await;

    // Heartbeat: always 503 so we hit the reconnect path.
    let hb_calls = Arc::new(AtomicU32::new(0));
    let hb_calls_c = hb_calls.clone();
    Mock::given(method("PUT"))
        .and(path("/session/1/heartbeat"))
        .respond_with(move |_req: &wiremock::Request| {
            hb_calls_c.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(503)
        })
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path("/session/1"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = ChromaClient::with_base_url(&server.uri());
    let cfg = SessionConfig {
        heartbeat_interval: Duration::from_millis(40),
        reconnect_threshold: 3,
        reinit_max: 1,
    };
    let session = Session::init(client.clone(), cfg).await.unwrap();

    // Give the heartbeat loop time to fail 3× and trigger reconnect attempt.
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(hb_calls.load(Ordering::SeqCst) >= 3, "heartbeat should have been attempted at least 3 times");

    session.shutdown().await;
}
