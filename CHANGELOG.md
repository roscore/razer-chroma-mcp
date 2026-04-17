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
