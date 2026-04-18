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

## M4 — <YYYY-MM-DD> (template, fill on live run)

**Environment**
- Host: Windows 11, rustc 1.94.1, `cargo build` debug profile
- Binary: `target/debug/razer-chroma-mcp.exe`
- Razer Synapse: running, Chroma SDK module enabled
- Razer hardware: BlackWidow (6×22 grid)
- Test harness: scripted MCP client (NDJSON stdio) driving `serve` subcommand

**Scenarios**

| # | Call | Expected visual | Pass? | Notes |
| - | ---- | --------------- | ----- | ----- |
| 1 | `signal_thinking` | Radial pulse from keyboard center, terracotta↔cream, ~2.8 s cycle, 35–65% brightness | | |
| 2 | `signal_needs_input` | 3× cream flash in first 900 ms (≥90% brightness), then TERRACOTTA breathing 1.2 s cycle with spacebar + Enter + Esc cream accents on 0.75 s counter-rhythm | | Does it feel "강렬하게 인상"? |
| 3 | `signal_idle` | Deep ember slow breathing, 12 s cycle, 5–12% brightness | | Uses built-in CHROMA_BREATHING (one PUT) |
| 4 | `signal_complete` (during Thinking) | 900 ms terracotta sweep L→R → 180 ms central cream asterisk flash → 400 ms fade → automatic restore to Thinking | | Asterisk visibly legible? |
| 5 | `signal_error` | Burnt rust slow breathing, 2.5 s cycle, 40–80% brightness | | |
| 6 | `set_static_color #D97757` after signal | Keyboard fills terracotta, state becomes UserControlled; signals cancel animator | | |
| 7 | Idle auto-entry (`RAZER_CHROMA_IDLE_MS=2000`, wait 3 s after UserControlled) | Transitions to Idle (deep ember breathing) automatically | | Eligibility: UserControlled + Error only |
| 8 | Reconnect restore (toggle Synapse / pull cable, then restore) | After `session reconnected` tracing log, the last persistent state re-appears (or cached user frame re-applied if UserControlled) | | Covers T4.13 |

**Observations**

<fill in after run>

**Carryovers**

<fill in>

## M5 — Reconnect-restore verification (T5.6)

Exercises `Session::reconnect()` + `StatefulController::restore_after_reconnect()`
end-to-end. The controller registers a watcher on the session's
`reconnected: Arc<Notify>` (see `src/chroma/session.rs`,
`src/signals/controller.rs`) that replays the last persistent state — or the
last user frame if the snapshot was `UserControlled` — every time reconnect
succeeds.

**Environment**
- Host: Windows 11, rustc 1.94.1, `cargo build` debug profile
- Binary: `target/debug/razer-chroma-mcp.exe`
- Razer Synapse: running, Chroma SDK module enabled
- Razer hardware: BlackWidow (6×22 grid)
- Log tail: `powershell -c "Get-Content -Wait $env:LOCALAPPDATA\razer-chroma-mcp\logs\mcp.log"`

**Forcing a reconnect.** Pick whichever is least invasive; all three route
through the same code path (`heartbeat` fails → `Reconnecting { failures }`
→ `reconnect()` → `notify_waiters()`):

- Close + reopen Razer Synapse (Synapse exit → Chroma REST drops → restart
  Synapse → watcher fires).
- Disable + re-enable the "Chroma Apps" toggle in Synapse. Faster than a full
  restart; stays within a single app instance.
- Temporarily block port 54235 with a firewall rule (`New-NetFirewallRule`
  then remove). Harshest — forces ≥3 heartbeat failures before reconnect.

**Scenarios**

| # | Pre-state | Action | Expected after reconnect | Pass? | Notes |
| - | --------- | ------ | ------------------------ | ----- | ----- |
| R1 | `signal_error` → Error breathing (burnt rust) | Close Synapse, wait until `Reconnecting` appears in logs, reopen Synapse | Error breathing resumes automatically; no manual re-trigger needed | | Error is a persistent state — `last_persistent = Error` |
| R2 | `signal_idle` → Idle breathing (deep ember) | Same as R1 | Idle breathing resumes | | `last_persistent = Idle` |
| R3 | `set_static_color #D97757` → UserControlled solid terracotta | Same as R1 | UserControlled frame re-PUT; keyboard fills terracotta again | | Replay path: cached `last_user_frame` |
| R4 | `signal_complete` just played (→ Off) | Same as R1 | Keyboard stays dark; no spurious pattern | | Off is a no-op branch in `restore_after_reconnect` |
| R5 | `signal_thinking` running (transient Thinking) | Same as R1 | Restores `last_persistent` (whichever it was — **not** Thinking). If the session started from a clean Off, keyboard stays dark. | | Thinking/NeedsInput/Complete are not persistent |

**What to look for in the logs**

- `heartbeat failure: ...` followed by `reconnect succeeded, session reinitialized`
  (success path) or `reconnect failed, marking Degraded` (exhausted retries).
- `restore_after_reconnect` tracing spans with the replayed variant.
- No `guard_healthy` rejections on the replayed transition (the restore path
  bypasses the Degraded guard because the session just came back Healthy).

**Observations**

<fill in after run>

**Carryovers**

<fill in>

