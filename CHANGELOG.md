# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Cargo project scaffolding (Rust 2021, MSRV 1.75).
- Module skeleton: `chroma`, `signals`, `mcp`, `cli`, `config`.
- `ChromaError` type with bilingual messages.
- `clap`-based CLI with `serve`/`setup`/`doctor`/`demo`/`pattern`/`uninstall` subcommand stubs.
- GitHub Actions CI (Windows build+test, macOS/Linux build-only portability).
- MIT + Apache-2.0 dual license.
- Chroma HTTP client (`ChromaClient`) with 2s timeout, `SynapseNotRunning` classification.
- `Session` lifecycle: 5s heartbeat, 3-fail reconnect threshold, graceful shutdown.
- Effect payloads (`CHROMA_NONE`/`CHROMA_STATIC`/`CHROMA_CUSTOM`/`CHROMA_BREATHING`/`CHROMA_WAVE`/`CHROMA_REACTIVE`/`CHROMA_SPECTRUMCYCLING`) with insta snapshot tests.
- `demo` subcommand — 3-second terracotta sweep on keyboard (live hardware verified).
- `rmcp` 1.5 stdio server exposing `set_static_color` / `clear_all` /
  `get_session_status` tools (NDJSON framing, typed input + output schemas).
- `Controller` with lazy `OnceCell`-gated session init, shared across tool
  handlers so the Chroma session is only spun up on first effect call.
- Dual-layer `tracing` logger: stderr + daily-rotated file at
  `%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log` (falls back to `./logs`).
- `tests/mcp_subprocess.rs` — spawns the built binary and drives the full
  MCP handshake + `tools/call` over NDJSON against a wiremock-backed Synapse.
- `docs/manual-testing.md` — live end-to-end smoke log for milestones that
  need real hardware + Synapse.
- Anthropic warm palette (`TERRACOTTA` / `WARM_CREAM` / `DEEP_EMBER` / `BURNT_RUST`)
  with `lerp` and `scale` helpers. Spec §5.1.
- `ClaudeState` / `PersistentState` enums with type-level
  Complete-never-nested invariant. Spec §5.2.
- Full state transition table (spec §5.3) covered by 42 unit tests.
- Animator runtime at 20 FPS with graceful (≤200 ms) / force cancel, and
  a one-shot completion `Notify`. Spec §5.7.
- Pattern generators: `thinking` (radial pulse), `needs_input`
  (3-stage flash + breathing + counter-rhythm accents, spec §5.4
  acceptance criterion encoded in tests), `idle` (deep ember breathing),
  `complete` (sweep + asterisk + fade, spec §5.5 asterisk mask),
  `error` (burnt-rust slow breathing).
- `signal_thinking` / `signal_needs_input` / `signal_idle` /
  `signal_complete` / `signal_error` high-level MCP tools.
- Low-level MCP surface completed: `set_static_color`, `set_custom_grid`,
  `play_preset_effect`, `clear_device`, `clear_all`, `get_device_info`,
  `get_session_status` — 12 tools total.
- Idle auto-entry (spec §5.9) with `RAZER_CHROMA_IDLE_MS` env override
  (default 10 min) and the UserControlled / Error eligibility rule.
- Reconnect restores the previous persistent state (or replays the
  cached user frame when UserControlled). Spec §5.8.

### Changed
- MSRV bumped 1.75 → 1.85 to adopt `rmcp` 1.5 (Rust edition 2024).
- MCP handlers retargeted from `Controller` to `StatefulController`,
  which owns the ClaudeState machine, animator lifecycle, inactivity
  timer, and reconnect-restore watcher.

### Fixed
- `heartbeat` PUT now sends an empty JSON body; Synapse's Chroma REST
  endpoint returns `HTTP 411 Length Required` for bodyless PUTs, which
  previously dragged long-lived sessions into `Reconnecting` → `Degraded`
  (surfaced by the M3 live smoke run).
