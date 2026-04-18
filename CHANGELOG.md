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

### Changed
- MSRV bumped 1.75 → 1.85 to adopt `rmcp` 1.5 (Rust edition 2024).

### Fixed
- `heartbeat` PUT now sends an empty JSON body; Synapse's Chroma REST
  endpoint returns `HTTP 411 Length Required` for bodyless PUTs, which
  previously dragged long-lived sessions into `Reconnecting` → `Degraded`
  (surfaced by the M3 live smoke run).
