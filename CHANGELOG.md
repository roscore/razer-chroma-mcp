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
