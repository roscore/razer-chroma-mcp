# Manual Testing Log

End-to-end smoke results against live hardware + Razer Synapse. Automated
unit/integration coverage lives under `cargo test`; this file records the
out-of-band hardware/IPC runs that cannot be reproduced in CI.

## M3 — 2026-04-18

**Environment**
- Host: Windows 11, rustc 1.94.1, `cargo build` debug profile
- Binary: `target/debug/razer-chroma-mcp.exe` (14.7 MB)
- Razer Synapse: running, Chroma SDK module enabled
- Razer hardware: keyboard (from M2 `demo` run — same device)
- Test harness: `C:\tmp\mcp_live_test.mjs` (Node 20), driving the binary over
  NDJSON stdio (same framing the Claude Desktop client uses). GUI-based
  verification via Claude Desktop was substituted with a scripted MCP client
  because the two paths share the same transport and tool surface; the config
  entry (`%APPDATA%\Claude\claude_desktop_config.json → mcpServers.razer-chroma`)
  was installed for day-to-day use.

**Scenario**

1. `initialize` handshake → expect serverInfo.
2. `notifications/initialized`.
3. `tools/call set_static_color { color: "#D97757" }` — fill keyboard terracotta.
4. Dwell 12 s (crosses the 5 s heartbeat cycle at least twice).
5. `tools/call get_session_status` — expect `initialized=true, state="Healthy"`.
6. `tools/call clear_all` — fill keyboard CHROMA_NONE.
7. `tools/call get_session_status` — still `Healthy`.

**Results**

| Step | Outcome | Notes |
| ---- | ------- | ----- |
| initialize | PASS | `serverInfo={name:"rmcp", version:"1.5.0"}` |
| set_static_color #D97757 | PASS | `{"status":"ok"}`, keyboard filled terracotta |
| get_session_status after 12 s | PASS | `{"initialized":true,"state":"Healthy"}` |
| clear_all | PASS | `{"status":"ok"}`, keyboard cleared |
| get_session_status post-clear | PASS | `{"initialized":true,"state":"Healthy"}` |

**Observations / issues uncovered**

- **Heartbeat 411 (fixed in this milestone).** First live run left the session
  stuck at `Reconnecting { failures: 1 }`. Stderr showed
  `heartbeat failure: HTTP 411 Length Required`. Synapse's Chroma REST
  endpoint rejects bodyless `PUT /heartbeat`; the fix is to send an explicit
  empty JSON object. Without this, any session kept open past `5 s × 3 =
  15 s` would cascade Reconnecting → Degraded. See `src/chroma/client.rs::heartbeat`.
- Re-run after the fix: session stayed `Healthy` across 12 s dwell with no
  411 warnings in stderr.
- MCP tool round-trip latency on cold path (first `set_static_color`, which
  triggers `Session::init`): ~900 ms on this box. Subsequent calls are sub-100 ms.
- No stray stdout noise detected — structured JSON lines only, as the NDJSON
  transport requires. Tracing output goes to stderr + the rolling file logger
  at `%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log`.

**Next-milestone carryovers (M4 / review backlog)**

- `get_session_status.state` is produced via `format!("{status:?}")` — fine for
  now, but M4 should stabilise the string tags before any client starts
  pattern-matching on them.
- `Controller::default()` hard-codes `SessionConfig::default()`; inject via
  `Controller::new` in M4 so tests (and the future `doctor` subcommand) can
  tune heartbeat/reinit without env vars.
