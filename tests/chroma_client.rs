//! Integration test for ChromaClient against a mocked Synapse REST endpoint.

use razer_chroma_mcp::chroma::client::ChromaClient;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn init_session_returns_session_id_and_uri() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "sessionid": 42,
        "uri": format!("{}/session/42", server.uri()),
    });
    Mock::given(method("POST"))
        .and(path("/razer/chromasdk"))
        .and(header("content-type", "application/json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;

    let client = ChromaClient::with_base_url(&server.uri());
    let session = client.init_session(init_payload()).await.unwrap();
    assert_eq!(session.session_id, 42);
    assert!(session.uri.ends_with("/session/42"));
}

fn init_payload() -> serde_json::Value {
    serde_json::json!({
        "title": "razer-chroma-mcp",
        "description": "test",
        "author": { "name": "roscore", "contact": "https://github.com/roscore" },
        "device_supported": ["keyboard"],
        "category": "application",
    })
}
