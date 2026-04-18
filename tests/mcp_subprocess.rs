//! End-to-end: spawn the built binary as a subprocess and drive it via
//! a minimal hand-rolled MCP JSON-RPC client over stdio. This avoids
//! depending on rmcp's client API surface.
//!
//! Framing note: rmcp 1.5 uses newline-delimited JSON on stdio (NOT the
//! LSP Content-Length header style). This test writes `<json>\n` per
//! message and reads a single line per message.

use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn send<W: AsyncWriteExt + Unpin>(w: &mut W, msg: Value) {
    let mut s = msg.to_string();
    s.push('\n');
    w.write_all(s.as_bytes()).await.unwrap();
    w.flush().await.unwrap();
}

async fn recv<R: tokio::io::AsyncRead + Unpin>(r: &mut BufReader<R>) -> Value {
    let mut line = String::new();
    r.read_line(&mut line).await.unwrap();
    assert!(!line.is_empty(), "server closed stdout before sending a frame");
    serde_json::from_str(line.trim_end()).expect("server returned invalid JSON line")
}

#[tokio::test(flavor = "multi_thread")]
async fn get_session_status_round_trip() {
    let server = MockServer::start().await;
    // We do not require the session to init successfully — `get_session_status`
    // returns even when the session has never been initialized.
    Mock::given(method("POST"))
        .and(path("/razer/chromasdk"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let bin = env!("CARGO_BIN_EXE_razer-chroma-mcp");
    let mut child = Command::new(bin)
        .arg("serve")
        .env("RAZER_CHROMA_BASE_URL", server.uri())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // MCP initialize handshake
    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "0" }
            }
        }),
    )
    .await;
    // 15s covers cold-start of the debug binary on a loaded CI runner.
    let _init = tokio::time::timeout(Duration::from_secs(15), recv(&mut stdout))
        .await
        .expect("initialize response timeout");

    // MCP required: notifications/initialized after initialize response
    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }),
    )
    .await;

    // tools/call get_session_status
    send(
        &mut stdin,
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "get_session_status", "arguments": {} }
        }),
    )
    .await;
    let resp = tokio::time::timeout(Duration::from_secs(15), recv(&mut stdout))
        .await
        .expect("tools/call response timeout");
    let result = resp.pointer("/result").expect("result present");
    // The tool should not have returned an error envelope.
    assert_ne!(
        result.pointer("/isError"),
        Some(&serde_json::Value::Bool(true)),
        "tool call returned isError=true; full body: {result}"
    );
    // The session was never initialized (mock Synapse returns 500 on init).
    // get_session_status must therefore report initialized=false in structuredContent.
    let initialized = result
        .pointer("/structuredContent/initialized")
        .unwrap_or_else(|| panic!("structuredContent.initialized missing; body: {result}"));
    assert_eq!(
        initialized,
        &serde_json::Value::Bool(false),
        "expected initialized=false; got body: {result}"
    );

    tokio::process::Child::kill(&mut child).await.unwrap();
    let _ = child.wait().await;
}
