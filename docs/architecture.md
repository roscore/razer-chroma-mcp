# Architecture

For contributors. User-facing docs are in the [README](../README.md).

## Three layers

```
mcp  →  signals  →  chroma
```

Dependencies are strictly one-way. `chroma` knows about HTTP and Chroma's
payload shapes but nothing about Claude states. `signals` owns
`ClaudeState` and animator lifecycles. `mcp` translates JSON-RPC into
`signals::controller` calls.

## State machine

`signals::controller::next_state` is a pure function of
`(from, incoming) → to`. Every cell of the 7×6 table is unit-tested
(42 tests). See spec §5.3 for the table.

The stateful controller wraps `next_state` with:

- A `Mutex<ControllerInner>` that serializes transitions.
- An animator handle that gets gracefully cancelled (≤200 ms) on
  transition.
- A `last_user_frame` cache used to restore `UserControlled` on
  reconnect.
- An inactivity timer that fires `Incoming::Idle` after
  `RAZER_CHROMA_IDLE_MS` (default 10 min) of no activity —
  eligible only from `UserControlled` and `Error`.

## Animator

Each pattern implements `Pattern::next_frame(session, tick)`. The
runtime ticks at 20 FPS (50 ms) and pushes the returned effect via
HTTP. Returning `Ok(false)` signals one-shot completion (used by
`complete`).

Cancellation modes:

| Scenario                     | Mode                                           |
|------------------------------|------------------------------------------------|
| State transition             | Graceful: signal + wait ≤200 ms + force-drop   |
| Heartbeat failure / shutdown | Force: abort the task immediately              |

## Session + reconnect restore

`Session::init` does a single `POST /razer/chromasdk` and spawns the
heartbeat task. Three consecutive heartbeat failures flip the session
to `Reconnecting`; after 5 reinit attempts we land on `Degraded` and
tool handlers reject writes with `SessionInitFailed`.

On successful reinit, `Session::reconnect` notifies waiters on an
`Arc<Notify>`. The `StatefulController` holds a watcher per session
that fires `restore_after_reconnect` — replaying either the last
persistent state (via `force_transition`) or the cached user frame
(when the snapshot was `UserControlled`).

## Why rmcp?

Official Anthropic Rust SDK. If upstream breaks compat we fall back
to manual JSON-RPC; see Task 3.1 of the plan for the decision record.

## Testing strategy

- Unit tests for every pure function (transition table, pattern
  frames, merge algorithm, hex parsing).
- wiremock integration tests for HTTP paths (session, heartbeat
  reconnect, 404-vs-503).
- Subprocess integration test for MCP stdio round-trip.
- Manual hardware tests in `docs/manual-testing.md` gate every
  release.
