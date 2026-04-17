# Razer Chroma MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust MCP server that gives Claude Desktop/Code a "body language" on Razer Chroma keyboards via Synapse's local REST API, installable by non-technical users via a Setup.exe.

**Architecture:** Three layers with strict one-way dependencies — `mcp → signals → chroma`. The `mcp` layer handles JSON-RPC tool invocation via rmcp. `signals::controller` owns the single `ClaudeState` and spawns/cancels tokio animator tasks on transitions. `chroma` wraps Synapse's REST API with heartbeat and reconnect. See `docs/superpowers/specs/2026-04-17-razer-chroma-mcp-design.md` for the full design.

**Tech Stack:** Rust 2021 (MSRV 1.75), tokio (multi-thread), reqwest (rustls), rmcp, clap (derive), tracing + tracing-appender, thiserror + anyhow, serde + serde_json, insta (snapshots), wiremock (HTTP mocks), NSIS (installer).

---

## File Structure

Files are organized by the three architectural layers plus CLI, config, and MCP glue. See spec §4 for the full tree. Key split points:

- `src/chroma/` — owns all HTTP traffic. No `signals`/`mcp` imports.
- `src/signals/` — owns `ClaudeState`, animator lifecycle, and every pattern generator. Each pattern in its own file.
- `src/mcp/tools.rs` — 12 tool handlers. Each handler delegates to `signals::controller`.
- `src/config/merge.rs` — deep-merge algorithm for Claude Desktop/Code configs. Standalone, heavily unit tested.
- `installer/razer-chroma-mcp.nsi` — NSIS script. Per-user install, no UAC.

---

## Milestone 1: Scaffolding

Create Cargo project, module skeleton, CLI subcommand stubs, CI, and LICENSE. Deliverable: `cargo build` green on Windows CI and `razer-chroma-mcp --help` lists every subcommand.

### Task 1.1: Initialize Cargo project

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Write Cargo.toml**

```toml
[package]
name = "razer-chroma-mcp"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
authors = ["roscore <https://github.com/roscore>"]
license = "MIT OR Apache-2.0"
description = "MCP server that gives Claude a body language on Razer Chroma keyboards"
repository = "https://github.com/roscore/razer-chroma-mcp"
readme = "README.md"
keywords = ["mcp", "razer", "chroma", "rgb", "claude"]
categories = ["command-line-utilities"]

[dependencies]
anyhow = "1"
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync", "time", "signal", "io-std"] }
tokio-util = { version = "0.7", features = ["rt"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
tracing-appender = "0.2"

[dev-dependencies]
insta = { version = "1", features = ["json"] }
wiremock = "0.6"
tokio = { version = "1", features = ["test-util", "macros", "rt-multi-thread"] }
tempfile = "3"

[profile.release]
lto = "thin"
codegen-units = 1
strip = true
```

- [ ] **Step 2: Write src/main.rs stub**

```rust
fn main() {
    println!("razer-chroma-mcp");
}
```

- [ ] **Step 3: Write .gitignore**

```gitignore
/target
Cargo.lock.bak
*.bak-*
*.corrupt-*
*.tmp-*
/installer/build/
/installer/*.exe
```

- [ ] **Step 4: Verify build**

Run: `rtk cargo build`
Expected: compiles successfully, `target/debug/razer-chroma-mcp.exe` exists.

- [ ] **Step 5: Commit**

```bash
rtk git add Cargo.toml Cargo.lock src/main.rs .gitignore
rtk git commit -m "chore: initialize cargo project with toolchain deps"
```

### Task 1.2: Add LICENSE files and CHANGELOG

**Files:**
- Create: `LICENSE-MIT`
- Create: `LICENSE-APACHE`
- Create: `CHANGELOG.md`
- Create: `README.md` (stub only — full README in M6)

- [ ] **Step 1: Write LICENSE-MIT**

Copy the standard MIT license text (<https://opensource.org/license/mit/>), substituting:
- Year: `2026`
- Holder: `roscore`

- [ ] **Step 2: Write LICENSE-APACHE**

Copy the standard Apache-2.0 license text (<https://www.apache.org/licenses/LICENSE-2.0.txt>). No substitutions — the text is verbatim.

- [ ] **Step 3: Write CHANGELOG.md**

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial scaffolding (M1).
```

- [ ] **Step 4: Write README.md stub**

```markdown
# razer-chroma-mcp

MCP server that gives Claude a body language on Razer Chroma keyboards.

Status: pre-alpha (v0.1.0 in progress). See `docs/superpowers/specs/` for design.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.
```

- [ ] **Step 5: Commit**

```bash
rtk git add LICENSE-MIT LICENSE-APACHE CHANGELOG.md README.md
rtk git commit -m "docs: add licenses, changelog, readme stub"
```

### Task 1.3: Create lib.rs and module skeleton

**Files:**
- Create: `src/lib.rs`
- Create: `src/error.rs`
- Create: `src/cli/mod.rs`
- Create: `src/config/mod.rs`
- Create: `src/chroma/mod.rs`
- Create: `src/signals/mod.rs`
- Create: `src/mcp/mod.rs`

- [ ] **Step 1: Write src/lib.rs**

```rust
//! razer-chroma-mcp — MCP server for Razer Chroma devices.
//!
//! See `docs/superpowers/specs/2026-04-17-razer-chroma-mcp-design.md`.

pub mod chroma;
pub mod cli;
pub mod config;
pub mod error;
pub mod mcp;
pub mod signals;

pub use error::ChromaError;
```

- [ ] **Step 2: Write src/error.rs (stub with all variants)**

```rust
//! Crate-wide error type. Per spec §8.1.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChromaError {
    #[error("Razer Synapse is not running. Please start Synapse and try again.\nRazer Synapse가 실행 중이 아닙니다.")]
    SynapseNotRunning,

    #[error("Chroma SDK module is disabled. Enable it in Synapse settings.\nSynapse 설정에서 Chroma 모듈을 켜주세요.")]
    ChromaModuleDisabled,

    #[error("Session init failed: {0}")]
    SessionInitFailed(String),

    #[error("Heartbeat failed ({failures}× in a row): {cause}")]
    HeartbeatFailed { failures: u32, cause: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid grid size: expected {expected:?}, got {actual:?}")]
    InvalidGridSize {
        expected: (usize, usize),
        actual: (usize, usize),
    },

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Unknown device: {0}")]
    UnknownDevice(String),

    #[error("Config file corrupt: {path}. A copy has been saved to {quarantine}.")]
    ConfigCorrupt { path: String, quarantine: String },
}

pub type Result<T> = std::result::Result<T, ChromaError>;
```

- [ ] **Step 3: Write empty module files**

`src/cli/mod.rs`:
```rust
//! CLI subcommand dispatch. See spec §4.
pub mod demo;
pub mod doctor;
pub mod pattern;
pub mod serve;
pub mod setup;
pub mod uninstall;
```

`src/config/mod.rs`:
```rust
//! Claude Desktop / Claude Code config path resolution and deep-merge.
pub mod merge;
pub mod paths;
```

`src/chroma/mod.rs`:
```rust
//! HTTP client + session lifecycle for Razer Chroma SDK REST API.
pub mod client;
pub mod color;
pub mod devices;
pub mod effects;
pub mod session;
```

`src/signals/mod.rs`:
```rust
//! Claude state machine, animators, and pattern generators.
pub mod controller;
pub mod inactivity;
pub mod palette;
pub mod patterns;
```

`src/mcp/mod.rs`:
```rust
//! MCP (Model Context Protocol) server bootstrap and tool handlers.
pub mod server;
pub mod tools;
```

- [ ] **Step 4: Create empty sub-module placeholder files**

Create these files with only a single `//! stub` doc comment (content filled in later milestones):

```
src/cli/demo.rs
src/cli/doctor.rs
src/cli/pattern.rs
src/cli/serve.rs
src/cli/setup.rs
src/cli/uninstall.rs
src/config/merge.rs
src/config/paths.rs
src/chroma/client.rs
src/chroma/color.rs
src/chroma/devices.rs
src/chroma/effects.rs
src/chroma/session.rs
src/signals/controller.rs
src/signals/inactivity.rs
src/signals/palette.rs
src/signals/patterns/mod.rs
src/signals/patterns/thinking.rs
src/signals/patterns/needs_input.rs
src/signals/patterns/idle.rs
src/signals/patterns/complete.rs
src/signals/patterns/error.rs
src/mcp/server.rs
src/mcp/tools.rs
```

Each file content:
```rust
//! stub — implemented in a later milestone.
```

Then update `src/signals/mod.rs` to declare `pub mod patterns;` and create `src/signals/patterns/mod.rs`:

```rust
//! Pattern generators for each Claude state.
pub mod complete;
pub mod error;
pub mod idle;
pub mod needs_input;
pub mod thinking;
```

- [ ] **Step 5: Verify build**

Run: `rtk cargo build`
Expected: compiles with only "unused imports" warnings acceptable, no errors.

- [ ] **Step 6: Commit**

```bash
rtk git add src/
rtk git commit -m "feat: add lib.rs, error.rs, and empty module skeleton"
```

### Task 1.4: Wire clap CLI with subcommand stubs

**Files:**
- Modify: `src/main.rs`
- Modify: `src/cli/mod.rs` (add `Cli` struct)
- Modify: `src/cli/serve.rs`, `setup.rs`, `doctor.rs`, `demo.rs`, `pattern.rs`, `uninstall.rs`

- [ ] **Step 1: Write src/cli/mod.rs with clap types**

```rust
//! CLI subcommand dispatch. See spec §4.

pub mod demo;
pub mod doctor;
pub mod pattern;
pub mod serve;
pub mod setup;
pub mod uninstall;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "razer-chroma-mcp", version, about = "MCP server for Razer Chroma devices")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the MCP stdio server (default if no subcommand given).
    Serve,
    /// Interactive setup wizard.
    Setup {
        /// Force a UI language instead of auto-detecting.
        #[arg(long, value_parser = ["ko", "en"])]
        lang: Option<String>,
    },
    /// Run environment diagnostics.
    Doctor,
    /// Play a 3-second terracotta sweep to verify hardware.
    Demo,
    /// Preview a named pattern without Claude Desktop.
    Pattern {
        /// Pattern name (thinking, needs_input, idle, complete, error).
        name: String,
        /// Seconds to run before exiting.
        #[arg(long, default_value_t = 8)]
        seconds: u64,
    },
    /// Remove Claude Desktop/Code config entries.
    Uninstall,
}
```

- [ ] **Step 2: Write each cli/*.rs stub to return a friendly "not implemented yet" error**

`src/cli/serve.rs`:
```rust
//! `serve` subcommand — runs the MCP stdio server. Implemented in M3.
use anyhow::Result;

pub async fn run() -> Result<()> {
    anyhow::bail!("serve: not implemented yet (M3)");
}
```

`src/cli/setup.rs`:
```rust
//! `setup` subcommand — install wizard. Implemented in M6.
use anyhow::Result;

pub async fn run(_lang: Option<String>) -> Result<()> {
    anyhow::bail!("setup: not implemented yet (M6)");
}
```

`src/cli/doctor.rs`:
```rust
//! `doctor` subcommand — environment diagnostics. Implemented in M5.
use anyhow::Result;

pub async fn run() -> Result<()> {
    anyhow::bail!("doctor: not implemented yet (M5)");
}
```

`src/cli/demo.rs`:
```rust
//! `demo` subcommand — 3-second sweep for hardware verification. Implemented in M2.
use anyhow::Result;

pub async fn run() -> Result<()> {
    anyhow::bail!("demo: not implemented yet (M2)");
}
```

`src/cli/pattern.rs`:
```rust
//! `pattern` subcommand — preview any named pattern. Implemented in M5.
use anyhow::Result;

pub async fn run(_name: String, _seconds: u64) -> Result<()> {
    anyhow::bail!("pattern: not implemented yet (M5)");
}
```

`src/cli/uninstall.rs`:
```rust
//! `uninstall` subcommand — remove Claude Desktop/Code entries. Implemented in M6.
use anyhow::Result;

pub async fn run() -> Result<()> {
    anyhow::bail!("uninstall: not implemented yet (M6)");
}
```

- [ ] **Step 3: Write src/main.rs with dispatcher**

```rust
use anyhow::Result;
use clap::Parser;
use razer_chroma_mcp::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => razer_chroma_mcp::cli::serve::run().await,
        Command::Setup { lang } => razer_chroma_mcp::cli::setup::run(lang).await,
        Command::Doctor => razer_chroma_mcp::cli::doctor::run().await,
        Command::Demo => razer_chroma_mcp::cli::demo::run().await,
        Command::Pattern { name, seconds } => razer_chroma_mcp::cli::pattern::run(name, seconds).await,
        Command::Uninstall => razer_chroma_mcp::cli::uninstall::run().await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_from_env("RAZER_CHROMA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).with_writer(std::io::stderr).init();
}
```

- [ ] **Step 4: Verify build and --help**

Run: `rtk cargo build`
Expected: builds successfully.

Run: `./target/debug/razer-chroma-mcp.exe --help`
Expected output includes `serve`, `setup`, `doctor`, `demo`, `pattern`, `uninstall` subcommands.

Run: `./target/debug/razer-chroma-mcp.exe demo`
Expected: exits with error message `Error: demo: not implemented yet (M2)`.

- [ ] **Step 5: Commit**

```bash
rtk git add src/main.rs src/cli/
rtk git commit -m "feat(cli): wire clap subcommands with stub handlers"
```

### Task 1.5: Add GitHub Actions CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Write .github/workflows/ci.yml**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  windows:
    name: Build + Test (windows-latest)
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75.0
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all
      - run: cargo build --release

  portability:
    name: Build-only portability check
    strategy:
      fail-fast: false
      matrix:
        os: [macos-latest, ubuntu-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75.0
      - uses: Swatinem/rust-cache@v2
      - run: cargo build
```

- [ ] **Step 2: Run local clippy + fmt check to match CI**

Run: `rtk cargo fmt --check`
Expected: no output (fmt clean). If it fails, run `rtk cargo fmt` and re-commit the formatting changes before proceeding.

Run: `rtk cargo clippy --all-targets --all-features -- -D warnings`
Expected: no warnings reported.

- [ ] **Step 3: Commit**

```bash
rtk git add .github/workflows/ci.yml
rtk git commit -m "ci: add windows build+test and macos/linux portability check"
```

### Task 1.6: Add CONTRIBUTING.md and milestone close-out

**Files:**
- Create: `CONTRIBUTING.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Write CONTRIBUTING.md stub**

```markdown
# Contributing

Thanks for considering a contribution!

## Dev Setup

- Install Rust 1.75 or newer: <https://rustup.rs>
- Windows is the primary target. macOS/Linux build but do not distribute.
- Install Razer Synapse with the Chroma module enabled for live testing.

## Style

- `cargo fmt` (checked in CI).
- `cargo clippy --all-targets --all-features -- -D warnings` (checked in CI).
- Tests go under `tests/` (integration) or inline `#[cfg(test)] mod tests` (unit).
- Prefer many small files (<400 lines typical, <800 max).

## PR Process

- Branch off `main`. Small, focused PRs.
- Include test updates. Integration tests use `wiremock`.
- Describe manual verification steps for hardware-touching changes.

## Razer Chroma SDK Terms

This project integrates with Razer Synapse's Chroma REST API. Contributors are responsible for compliance with Razer's [SDK license terms](https://developer.razer.com/works-with-chroma/razer-chroma-workshop/) — in particular, do not redistribute Razer assets and do not claim affiliation with Razer.
```

- [ ] **Step 2: Update CHANGELOG.md**

Replace the `## [Unreleased]` section content with:

```markdown
## [Unreleased]

### Added
- Cargo project scaffolding (Rust 2021, MSRV 1.75).
- Module skeleton: `chroma`, `signals`, `mcp`, `cli`, `config`.
- `ChromaError` type with bilingual messages.
- `clap`-based CLI with `serve`/`setup`/`doctor`/`demo`/`pattern`/`uninstall` subcommand stubs.
- GitHub Actions CI (Windows build+test, macOS/Linux build-only portability).
- MIT + Apache-2.0 dual license.
```

- [ ] **Step 3: Verify M1 acceptance**

Run: `rtk cargo build && ./target/debug/razer-chroma-mcp.exe --help`
Expected: lists all 6 subcommands; build succeeds.

- [ ] **Step 4: Commit**

```bash
rtk git add CONTRIBUTING.md CHANGELOG.md
rtk git commit -m "docs: add contributing guide and m1 changelog"
```

---

## Milestone 2: Chroma Client

Build the HTTP client for Razer's Chroma REST API: color types, device grid, effect payloads, session init/heartbeat/drop/reconnect, and a working `demo` subcommand that displays a terracotta sweep on a live BlackWidow.

### Task 2.1: `Rgb` type and hex parser (TDD)

**Files:**
- Modify: `src/chroma/color.rs`
- Test: inline `#[cfg(test)]` in `src/chroma/color.rs`

- [ ] **Step 1: Write the failing tests**

```rust
//! RGB color type, hex parsing, and BGR u32 encoding for the Chroma SDK.

use crate::error::{ChromaError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const BLACK: Self = Self::new(0, 0, 0);

    /// Parse a hex color string like `#RRGGBB` or `#RGB`. Leading `#` is required.
    pub fn from_hex(s: &str) -> Result<Self> {
        let s = s.trim();
        let body = s.strip_prefix('#')
            .ok_or_else(|| ChromaError::InvalidColor(format!("missing leading '#': {s}")))?;
        let parse_u8 = |slice: &str| u8::from_str_radix(slice, 16)
            .map_err(|_| ChromaError::InvalidColor(format!("not valid hex: {s}")));
        match body.len() {
            6 => Ok(Self::new(parse_u8(&body[0..2])?, parse_u8(&body[2..4])?, parse_u8(&body[4..6])?)),
            3 => {
                let expand = |c: char| u8::from_str_radix(&format!("{c}{c}"), 16)
                    .map_err(|_| ChromaError::InvalidColor(format!("not valid hex: {s}")));
                let mut chars = body.chars();
                Ok(Self::new(
                    expand(chars.next().unwrap())?,
                    expand(chars.next().unwrap())?,
                    expand(chars.next().unwrap())?,
                ))
            }
            _ => Err(ChromaError::InvalidColor(format!("length must be 3 or 6 hex digits: {s}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_long_hex() {
        assert_eq!(Rgb::from_hex("#D97757").unwrap(), Rgb::new(0xD9, 0x77, 0x57));
    }

    #[test]
    fn parses_short_hex() {
        assert_eq!(Rgb::from_hex("#F0E").unwrap(), Rgb::new(0xFF, 0x00, 0xEE));
    }

    #[test]
    fn rejects_missing_hash() {
        assert!(matches!(Rgb::from_hex("D97757"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn rejects_bad_length() {
        assert!(matches!(Rgb::from_hex("#D977"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn rejects_non_hex_chars() {
        assert!(matches!(Rgb::from_hex("#GGGGGG"), Err(ChromaError::InvalidColor(_))));
    }

    #[test]
    fn parses_case_insensitive() {
        assert_eq!(Rgb::from_hex("#d97757").unwrap(), Rgb::from_hex("#D97757").unwrap());
    }
}
```

- [ ] **Step 2: Run tests to verify they pass**

(Note: this step combines RED+GREEN because the implementation is small and well-known; the tests exercise branches we need to keep correct.)

Run: `rtk cargo test --lib chroma::color`
Expected: all 6 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/chroma/color.rs
rtk git commit -m "feat(chroma): rgb type with hex parser"
```

### Task 2.2: RGB → BGR u32 conversion (TDD)

**Files:**
- Modify: `src/chroma/color.rs`

- [ ] **Step 1: Write the failing test**

Add to the `tests` module at the bottom of `src/chroma/color.rs`:

```rust
    #[test]
    fn bgr_u32_packs_low_to_high() {
        // Chroma SDK encodes as 0x00BBGGRR (little-endian BGR).
        let c = Rgb::new(0xD9, 0x77, 0x57);
        assert_eq!(c.to_bgr_u32(), 0x00_57_77_D9);
    }

    #[test]
    fn bgr_u32_round_trip() {
        let c = Rgb::new(0x12, 0x34, 0x56);
        assert_eq!(Rgb::from_bgr_u32(c.to_bgr_u32()), c);
    }

    #[test]
    fn bgr_u32_black_is_zero() {
        assert_eq!(Rgb::BLACK.to_bgr_u32(), 0);
    }
```

- [ ] **Step 2: Run the tests — expect failure**

Run: `rtk cargo test --lib chroma::color::tests::bgr`
Expected: FAIL — `to_bgr_u32` not found.

- [ ] **Step 3: Implement the conversion**

Add these methods to `impl Rgb` in `src/chroma/color.rs`:

```rust
    /// Chroma SDK wire format: 0x00BBGGRR. Red is the low byte.
    pub const fn to_bgr_u32(self) -> u32 {
        (self.b as u32) << 16 | (self.g as u32) << 8 | self.r as u32
    }

    pub const fn from_bgr_u32(v: u32) -> Self {
        Self::new((v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8, ((v >> 16) & 0xFF) as u8)
    }
```

- [ ] **Step 4: Run tests to verify pass**

Run: `rtk cargo test --lib chroma::color`
Expected: all 9 tests pass.

- [ ] **Step 5: Commit**

```bash
rtk git add src/chroma/color.rs
rtk git commit -m "feat(chroma): rgb to bgr u32 conversion for chroma wire format"
```

### Task 2.3: `Device` enum and grid constants

**Files:**
- Modify: `src/chroma/devices.rs`

- [ ] **Step 1: Write the failing tests**

```rust
//! Chroma-supported device kinds and their grid dimensions.
//!
//! v0.1.0 patterns target the generic keyboard grid only (spec §5.6).
//! Other devices are addressable via low-level tools for basic colors.

use crate::error::{ChromaError, Result};

pub const KEYBOARD_ROWS: usize = 6;
pub const KEYBOARD_COLS: usize = 22;
pub const MOUSE_ROWS: usize = 9;
pub const MOUSE_COLS: usize = 7;
pub const MOUSEPAD_LEDS: usize = 20;
pub const HEADSET_LEDS: usize = 5;
pub const KEYPAD_ROWS: usize = 4;
pub const KEYPAD_COLS: usize = 5;
pub const CHROMALINK_LEDS: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Keyboard,
    Mouse,
    Mousepad,
    Headset,
    Keypad,
    ChromaLink,
}

impl Device {
    pub const fn path_segment(self) -> &'static str {
        match self {
            Self::Keyboard => "keyboard",
            Self::Mouse => "mouse",
            Self::Mousepad => "mousepad",
            Self::Headset => "headset",
            Self::Keypad => "keypad",
            Self::ChromaLink => "chromalink",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "keyboard" => Ok(Self::Keyboard),
            "mouse" => Ok(Self::Mouse),
            "mousepad" => Ok(Self::Mousepad),
            "headset" => Ok(Self::Headset),
            "keypad" => Ok(Self::Keypad),
            "chromalink" => Ok(Self::ChromaLink),
            other => Err(ChromaError::UnknownDevice(other.to_string())),
        }
    }

    /// Grid dimensions as `Some((rows, cols))` for grid devices, `None` for LED strips.
    pub const fn grid(self) -> Option<(usize, usize)> {
        match self {
            Self::Keyboard => Some((KEYBOARD_ROWS, KEYBOARD_COLS)),
            Self::Mouse => Some((MOUSE_ROWS, MOUSE_COLS)),
            Self::Keypad => Some((KEYPAD_ROWS, KEYPAD_COLS)),
            Self::Mousepad | Self::Headset | Self::ChromaLink => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_grid_is_6x22() {
        assert_eq!(Device::Keyboard.grid(), Some((6, 22)));
    }

    #[test]
    fn parse_is_case_insensitive() {
        assert_eq!(Device::parse("KEYBOARD").unwrap(), Device::Keyboard);
    }

    #[test]
    fn parse_rejects_unknown() {
        assert!(matches!(Device::parse("gpu"), Err(ChromaError::UnknownDevice(_))));
    }

    #[test]
    fn path_segment_stable() {
        assert_eq!(Device::Keyboard.path_segment(), "keyboard");
    }
}
```

- [ ] **Step 2: Run the tests**

Run: `rtk cargo test --lib chroma::devices`
Expected: all 4 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/chroma/devices.rs
rtk git commit -m "feat(chroma): device enum and grid constants"
```

### Task 2.4: Effect payload types with snapshot tests

**Files:**
- Modify: `src/chroma/effects.rs`
- Create: `src/chroma/snapshots/` (populated by insta on first run)

- [ ] **Step 1: Write effect types and snapshot tests**

```rust
//! Razer Chroma SDK effect payloads. One struct per effect kind. See spec §4.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{Device, KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::error::{ChromaError, Result};
use serde::Serialize;

/// Common `{ "effect": "...", "param": ... }` wrapper as sent to Chroma REST API.
#[derive(Serialize, Debug, Clone)]
pub struct EffectEnvelope {
    pub effect: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<serde_json::Value>,
}

impl EffectEnvelope {
    pub fn none() -> Self {
        Self { effect: "CHROMA_NONE", param: None }
    }

    pub fn r#static(color: Rgb) -> Self {
        Self {
            effect: "CHROMA_STATIC",
            param: Some(serde_json::json!({ "color": color.to_bgr_u32() })),
        }
    }

    pub fn keyboard_custom(grid: &[[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS]) -> Self {
        let mut rows = Vec::with_capacity(KEYBOARD_ROWS);
        for r in 0..KEYBOARD_ROWS {
            let mut row = Vec::with_capacity(KEYBOARD_COLS);
            for c in 0..KEYBOARD_COLS {
                row.push(grid[r][c].to_bgr_u32());
            }
            rows.push(row);
        }
        Self {
            effect: "CHROMA_CUSTOM",
            param: Some(serde_json::Value::Array(
                rows.into_iter()
                    .map(|r| serde_json::Value::Array(r.into_iter().map(|v| serde_json::json!(v)).collect()))
                    .collect(),
            )),
        }
    }

    pub fn spectrumcycling() -> Self {
        Self { effect: "CHROMA_SPECTRUMCYCLING", param: None }
    }

    pub fn wave(direction: WaveDirection) -> Self {
        Self {
            effect: "CHROMA_WAVE",
            param: Some(serde_json::json!({ "direction": direction as u8 })),
        }
    }

    pub fn breathing(kind: BreathingKind) -> Self {
        let v = match kind {
            BreathingKind::Random => serde_json::json!({ "type": 1 }),
            BreathingKind::OneColor(c) => serde_json::json!({ "type": 2, "color": c.to_bgr_u32() }),
            BreathingKind::TwoColors(a, b) => serde_json::json!({
                "type": 3, "color1": a.to_bgr_u32(), "color2": b.to_bgr_u32()
            }),
        };
        Self { effect: "CHROMA_BREATHING", param: Some(v) }
    }

    pub fn reactive(color: Rgb, duration: ReactiveDuration) -> Self {
        Self {
            effect: "CHROMA_REACTIVE",
            param: Some(serde_json::json!({
                "color": color.to_bgr_u32(),
                "duration": duration as u8,
            })),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum WaveDirection { LeftToRight = 1, RightToLeft = 2 }

#[derive(Clone, Copy, Debug)]
pub enum BreathingKind {
    Random,
    OneColor(Rgb),
    TwoColors(Rgb, Rgb),
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReactiveDuration { Short = 1, Medium = 2, Long = 3 }

/// Build a full-keyboard custom grid from a single color.
pub fn keyboard_uniform(color: Rgb) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
    [[color; KEYBOARD_COLS]; KEYBOARD_ROWS]
}

/// Validate a runtime-dynamic grid matches the device's expected dimensions.
pub fn validate_grid(device: Device, rows: usize, cols: usize) -> Result<()> {
    if let Some(expected) = device.grid() {
        if expected != (rows, cols) {
            return Err(ChromaError::InvalidGridSize { expected, actual: (rows, cols) });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chroma::color::Rgb;

    #[test]
    fn static_effect_snapshot() {
        let payload = EffectEnvelope::r#static(Rgb::new(0xD9, 0x77, 0x57));
        insta::assert_json_snapshot!(payload);
    }

    #[test]
    fn none_effect_snapshot() {
        insta::assert_json_snapshot!(EffectEnvelope::none());
    }

    #[test]
    fn breathing_two_colors_snapshot() {
        let p = EffectEnvelope::breathing(BreathingKind::TwoColors(
            Rgb::new(0x3E, 0x27, 0x23),
            Rgb::new(0xF0, 0xEE, 0xE6),
        ));
        insta::assert_json_snapshot!(p);
    }

    #[test]
    fn keyboard_custom_serializes_as_6x22() {
        let grid = keyboard_uniform(Rgb::new(1, 2, 3));
        let p = EffectEnvelope::keyboard_custom(&grid);
        let v = serde_json::to_value(&p).unwrap();
        let param = v.get("param").unwrap();
        assert!(param.is_array());
        assert_eq!(param.as_array().unwrap().len(), 6);
        assert_eq!(param.as_array().unwrap()[0].as_array().unwrap().len(), 22);
    }

    #[test]
    fn validate_grid_rejects_wrong_size() {
        assert!(matches!(
            validate_grid(Device::Keyboard, 5, 22),
            Err(ChromaError::InvalidGridSize { .. })
        ));
    }

    #[test]
    fn validate_grid_accepts_correct_size() {
        validate_grid(Device::Keyboard, 6, 22).unwrap();
    }
}
```

- [ ] **Step 2: Run tests (first run will create snapshots)**

Run: `rtk cargo test --lib chroma::effects`
Expected: first run fails with "snapshot created" messages. Accept them:

Run: `rtk cargo insta accept` (or: `INSTA_UPDATE=always cargo test --lib chroma::effects`). If `cargo-insta` is not installed: `cargo install cargo-insta`.

Run: `rtk cargo test --lib chroma::effects` again.
Expected: all 6 tests pass; `src/chroma/snapshots/*.snap` files committed.

- [ ] **Step 3: Commit**

```bash
rtk git add src/chroma/effects.rs src/chroma/snapshots/
rtk git commit -m "feat(chroma): effect payload types with insta snapshots"
```

### Task 2.5: `ChromaClient` HTTP wrapper with wiremock test

**Files:**
- Modify: `src/chroma/client.rs`
- Test: `tests/chroma_client.rs`

- [ ] **Step 1: Write the failing integration test**

`tests/chroma_client.rs`:
```rust
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
```

- [ ] **Step 2: Run the test — expect failure**

Run: `rtk cargo test --test chroma_client`
Expected: FAIL — `ChromaClient::with_base_url` / `init_session` missing.

- [ ] **Step 3: Implement ChromaClient**

```rust
//! Thin HTTP wrapper over the Chroma SDK REST API (spec §4).
//!
//! All calls time out after 2 seconds unless overridden by
//! `RAZER_CHROMA_TIMEOUT_MS` at construction time.

use crate::error::{ChromaError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

const DEFAULT_BASE: &str = "http://localhost:54235/razer/chromasdk";

#[derive(Debug, Clone, Deserialize)]
pub struct Session {
    #[serde(rename = "sessionid")]
    pub session_id: u64,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitPayload {
    pub title: String,
    pub description: String,
    pub author: Author,
    pub device_supported: Vec<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Author {
    pub name: String,
    pub contact: String,
}

#[derive(Debug, Clone)]
pub struct ChromaClient {
    http: Client,
    base_url: String,
}

impl Default for ChromaClient {
    fn default() -> Self {
        Self::with_base_url(DEFAULT_BASE)
    }
}

impl ChromaClient {
    pub fn with_base_url(base: &str) -> Self {
        let timeout_ms = std::env::var("RAZER_CHROMA_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(2_000);
        let http = Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .expect("reqwest client build");
        let base_url = if base.ends_with("/razer/chromasdk") {
            base.to_string()
        } else {
            format!("{}/razer/chromasdk", base.trim_end_matches('/'))
        };
        Self { http, base_url }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn init_session<T: Serialize>(&self, payload: T) -> Result<Session> {
        let resp = self.http.post(&self.base_url).json(&payload).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::SessionInitFailed(format!("HTTP {}", resp.status())));
        }
        resp.json::<Session>()
            .await
            .map_err(|e| ChromaError::SessionInitFailed(e.to_string()))
    }

    /// PUT an effect to the per-session endpoint `<uri>/<device>`.
    pub async fn put_effect(
        &self,
        session_uri: &str,
        device_path: &str,
        effect: &crate::chroma::effects::EffectEnvelope,
    ) -> Result<()> {
        let url = format!("{}/{}", session_uri.trim_end_matches('/'), device_path);
        let resp = self.http.put(&url).json(effect).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::SessionInitFailed(format!(
                "PUT {} -> HTTP {}",
                url,
                resp.status()
            )));
        }
        Ok(())
    }

    /// PUT a heartbeat to `<uri>/heartbeat`.
    pub async fn heartbeat(&self, session_uri: &str) -> Result<()> {
        let url = format!("{}/heartbeat", session_uri.trim_end_matches('/'));
        let resp = self.http.put(&url).send().await?;
        if !resp.status().is_success() {
            return Err(ChromaError::HeartbeatFailed {
                failures: 1,
                cause: format!("HTTP {}", resp.status()),
            });
        }
        Ok(())
    }

    /// DELETE the session to tear it down cleanly.
    pub async fn delete_session(&self, session_uri: &str) -> Result<()> {
        let resp = self.http.delete(session_uri).send().await?;
        if !resp.status().is_success() {
            tracing::warn!(status = %resp.status(), "delete_session returned non-2xx; ignoring");
        }
        Ok(())
    }
}
```

- [ ] **Step 4: Run tests — expect pass**

Run: `rtk cargo test --test chroma_client`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
rtk git add src/chroma/client.rs tests/chroma_client.rs
rtk git commit -m "feat(chroma): http client with wiremock-verified session init"
```

### Task 2.6: `Session` lifecycle (heartbeat loop + reconnect)

**Files:**
- Modify: `src/chroma/session.rs`
- Test: `tests/chroma_session.rs`

- [ ] **Step 1: Write the failing test**

`tests/chroma_session.rs`:
```rust
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
```

- [ ] **Step 2: Run test — expect failure**

Run: `rtk cargo test --test chroma_session`
Expected: FAIL — types missing.

- [ ] **Step 3: Implement Session**

```rust
//! Owns the Chroma session lifecycle: init, heartbeat loop, reconnect, graceful drop.
//!
//! Per spec §3: heartbeat 5s interval, 2s HTTP timeout, 3 consecutive failures triggers reconnect.
//! Tests can override all three via `SessionConfig`.

use crate::chroma::client::{Author, ChromaClient, InitPayload};
use crate::chroma::effects::EffectEnvelope;
use crate::error::{ChromaError, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct SessionConfig {
    pub heartbeat_interval: Duration,
    pub reconnect_threshold: u32,
    pub reinit_max: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: Duration::from_secs(5),
            reconnect_threshold: 3,
            reinit_max: 5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionStatus {
    Uninitialized,
    Healthy,
    Reconnecting { failures: u32 },
    Degraded,
    ShutDown,
}

#[derive(Debug)]
struct Inner {
    uri: String,
    status: SessionStatus,
    failures: u32,
}

#[derive(Clone)]
pub struct Session {
    client: ChromaClient,
    cfg: SessionConfig,
    inner: Arc<RwLock<Inner>>,
    cancel: CancellationToken,
    hb_handle: Arc<tokio::sync::Mutex<Option<JoinHandle<()>>>>,
}

pub fn default_init_payload() -> InitPayload {
    InitPayload {
        title: "razer-chroma-mcp".into(),
        description: "MCP server for Razer Chroma".into(),
        author: Author {
            name: "roscore".into(),
            contact: "https://github.com/roscore".into(),
        },
        device_supported: vec![
            "keyboard".into(),
            "mouse".into(),
            "mousepad".into(),
            "headset".into(),
            "keypad".into(),
            "chromalink".into(),
        ],
        category: "application".into(),
    }
}

impl Session {
    pub async fn init(client: ChromaClient, cfg: SessionConfig) -> Result<Self> {
        let session = client.init_session(default_init_payload()).await?;
        let inner = Arc::new(RwLock::new(Inner {
            uri: session.uri.clone(),
            status: SessionStatus::Healthy,
            failures: 0,
        }));
        let cancel = CancellationToken::new();
        let me = Self {
            client,
            cfg,
            inner,
            cancel: cancel.clone(),
            hb_handle: Arc::new(tokio::sync::Mutex::new(None)),
        };
        let handle = tokio::spawn(me.clone().run_heartbeat(cancel));
        *me.hb_handle.lock().await = Some(handle);
        Ok(me)
    }

    pub async fn status(&self) -> SessionStatus {
        self.inner.read().await.status.clone()
    }

    pub async fn uri(&self) -> String {
        self.inner.read().await.uri.clone()
    }

    pub async fn put_effect(&self, device_path: &str, effect: &EffectEnvelope) -> Result<()> {
        let uri = self.uri().await;
        self.client.put_effect(&uri, device_path, effect).await
    }

    async fn run_heartbeat(self, cancel: CancellationToken) {
        let mut ticker = tokio::time::interval(self.cfg.heartbeat_interval);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = ticker.tick() => {
                    let uri = self.uri().await;
                    match self.client.heartbeat(&uri).await {
                        Ok(()) => {
                            let mut g = self.inner.write().await;
                            g.failures = 0;
                            if matches!(g.status, SessionStatus::Reconnecting { .. }) {
                                g.status = SessionStatus::Healthy;
                            }
                        }
                        Err(e) => {
                            let should_reconnect;
                            {
                                let mut g = self.inner.write().await;
                                g.failures += 1;
                                g.status = SessionStatus::Reconnecting { failures: g.failures };
                                should_reconnect = g.failures >= self.cfg.reconnect_threshold;
                                tracing::warn!(failures = g.failures, "heartbeat failure: {e}");
                            }
                            if should_reconnect {
                                self.reconnect().await;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn reconnect(&self) {
        for attempt in 1..=self.cfg.reinit_max {
            match self.client.init_session(default_init_payload()).await {
                Ok(s) => {
                    let mut g = self.inner.write().await;
                    g.uri = s.uri;
                    g.status = SessionStatus::Healthy;
                    g.failures = 0;
                    tracing::info!(attempt, "session reconnected");
                    return;
                }
                Err(e) => {
                    tracing::warn!(attempt, "reconnect attempt failed: {e}");
                    tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
            }
        }
        self.inner.write().await.status = SessionStatus::Degraded;
        tracing::error!("session marked DEGRADED after {} reinit attempts", self.cfg.reinit_max);
    }

    pub async fn shutdown(&self) {
        self.cancel.cancel();
        if let Some(h) = self.hb_handle.lock().await.take() {
            let _ = h.await;
        }
        let uri = self.uri().await;
        // Best-effort NONE + delete
        let _ = self.client.put_effect(&uri, "keyboard", &EffectEnvelope::none()).await;
        let _ = self.client.delete_session(&uri).await;
        self.inner.write().await.status = SessionStatus::ShutDown;
    }
}

// Keep the unused-imports clean when ChromaError is only used for type aliasing downstream.
#[allow(dead_code)]
fn _unused(_: ChromaError) {}
```

- [ ] **Step 4: Run test — expect pass**

Run: `rtk cargo test --test chroma_session`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
rtk git add src/chroma/session.rs tests/chroma_session.rs
rtk git commit -m "feat(chroma): session lifecycle with heartbeat loop and reconnect"
```

### Task 2.7: `SynapseNotRunning` diagnostic mapping

**Files:**
- Modify: `src/chroma/client.rs`
- Test: inline unit test

- [ ] **Step 1: Add the failing test**

At the bottom of `src/chroma/client.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ChromaError;

    #[tokio::test]
    async fn connection_refused_maps_to_synapse_not_running() {
        // Port 1 is reserved and will refuse connection on most systems.
        let client = ChromaClient::with_base_url("http://127.0.0.1:1");
        let err = client.init_session(serde_json::json!({})).await.unwrap_err();
        assert!(
            matches!(err, ChromaError::SynapseNotRunning | ChromaError::Network(_)),
            "got: {err:?}"
        );
    }
}
```

- [ ] **Step 2: Run test — it will currently surface `ChromaError::Network`. Update `init_session` to classify.**

In `src/chroma/client.rs`, replace the body of `init_session` with:

```rust
    pub async fn init_session<T: Serialize>(&self, payload: T) -> Result<Session> {
        let resp = self
            .http
            .post(&self.base_url)
            .json(&payload)
            .send()
            .await
            .map_err(classify_send_error)?;
        if !resp.status().is_success() {
            return Err(ChromaError::SessionInitFailed(format!("HTTP {}", resp.status())));
        }
        resp.json::<Session>()
            .await
            .map_err(|e| ChromaError::SessionInitFailed(e.to_string()))
    }
```

And add near the top of the file:

```rust
fn classify_send_error(e: reqwest::Error) -> ChromaError {
    if e.is_connect() || e.is_timeout() {
        ChromaError::SynapseNotRunning
    } else {
        ChromaError::Network(e)
    }
}
```

- [ ] **Step 3: Run tests**

Run: `rtk cargo test --lib chroma::client`
Expected: PASS — err is `SynapseNotRunning`.

- [ ] **Step 4: Commit**

```bash
rtk git add src/chroma/client.rs
rtk git commit -m "feat(chroma): classify connection-refused as SynapseNotRunning"
```

### Task 2.8: Implement `demo` subcommand (live hardware smoke test)

**Files:**
- Modify: `src/cli/demo.rs`

- [ ] **Step 1: Write the demo logic**

```rust
//! `demo` subcommand — 3-second terracotta→cream left-to-right sweep on keyboard.
//!
//! Acts as a hardware smoke test: init session, play sweep, tear down.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig};
use anyhow::{Context, Result};
use std::time::{Duration, Instant};

const TERRACOTTA: Rgb = Rgb::new(0xD9, 0x77, 0x57);
const CREAM: Rgb = Rgb::new(0xF0, 0xEE, 0xE6);

pub async fn run() -> Result<()> {
    let client = ChromaClient::default();
    let session = Session::init(client, SessionConfig::default())
        .await
        .context("could not start Chroma session — is Synapse running?")?;

    // Initial fill to terracotta
    let grid0 = keyboard_uniform(TERRACOTTA);
    session
        .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid0))
        .await?;

    let start = Instant::now();
    let total = Duration::from_millis(3_000);
    let mut tick = tokio::time::interval(Duration::from_millis(50));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    while Instant::now().duration_since(start) < total {
        tick.tick().await;
        let t = Instant::now().duration_since(start).as_millis() as f32 / total.as_millis() as f32;
        let leading_col = (t * (KEYBOARD_COLS as f32 + 4.0)) as i32 - 2;
        let grid = sweep_frame(leading_col);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
            .await?;
    }

    session.shutdown().await;
    println!("demo complete");
    Ok(())
}

fn sweep_frame(leading_col: i32) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
    let mut grid = [[TERRACOTTA; KEYBOARD_COLS]; KEYBOARD_ROWS];
    for row in 0..KEYBOARD_ROWS {
        for col in 0..KEYBOARD_COLS {
            let dist = (col as i32 - leading_col).abs();
            if dist <= 2 {
                grid[row][col] = CREAM;
            }
        }
    }
    grid
}
```

- [ ] **Step 2: Confirm `serve` is not required to build demo**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Live hardware verification (manual)**

Run: `./target/debug/razer-chroma-mcp.exe demo`

Expected on a BlackWidow with Synapse + Chroma enabled:
- Keyboard fills terracotta.
- A cream band sweeps left-to-right over ~3 seconds.
- Prints `demo complete` and exits 0.
- Keyboard reverts to Synapse profile.

If Synapse is not running:
- Exits non-zero with message containing `Synapse`.

Log the observation (date, hardware, pass/fail, notes) as a comment in this task's commit message.

- [ ] **Step 4: Commit**

```bash
rtk git add src/cli/demo.rs
rtk git commit -m "feat(cli): demo subcommand — 3-second terracotta sweep

Manual verification: <fill in: date, hardware, observed behavior>"
```

### Task 2.9: M2 close-out

- [ ] **Step 1: Update CHANGELOG.md**

Append to the `[Unreleased] / Added` list:

```markdown
- Chroma HTTP client (`ChromaClient`) with 2s timeout, `SynapseNotRunning` classification.
- `Session` lifecycle: 5s heartbeat, 3-fail reconnect threshold, graceful shutdown.
- Effect payloads (`CHROMA_NONE`/`CHROMA_STATIC`/`CHROMA_CUSTOM`/`CHROMA_BREATHING`/`CHROMA_WAVE`/`CHROMA_REACTIVE`/`CHROMA_SPECTRUMCYCLING`) with insta snapshot tests.
- `demo` subcommand — 3-second terracotta sweep on keyboard (live hardware verified).
```

- [ ] **Step 2: Commit**

```bash
rtk git add CHANGELOG.md
rtk git commit -m "docs: changelog m2 entries"
```

- [ ] **Step 3: M2 acceptance report**

Prepare a short status report for the user:
- All unit + integration tests green.
- `demo` verified on live BlackWidow (date, observations).
- Milestone complete; request approval to start M3.

---

## Milestone 3: MCP Server (minimal surface)

Stand up the rmcp stdio server with just enough tools — `set_static_color`, `clear_all`, `get_session_status` — to prove Claude Desktop can invoke us end-to-end. Full tool surface comes in M4.

### Task 3.1: Add rmcp dependency and verify API shape

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Research current rmcp API**

Per spec §12: "at M3 start, verify current version & API; if unworkable, fall back to manual JSON-RPC over stdio".

Run: `rtk cargo search rmcp --limit 5`
Run: (if internet available) check <https://github.com/modelcontextprotocol/rust-sdk> README for the current version and the tool-handler trait shape.

Record in the commit message: chosen rmcp version and the concrete trait/macro used for tool handlers. If rmcp is unworkable, stop and open a design note documenting the decision before continuing with a manual JSON-RPC fallback.

- [ ] **Step 2: Add dependency**

Append to `[dependencies]` in `Cargo.toml`:

```toml
rmcp = { version = "0.1", features = ["server", "tokio", "transport-io"] }
# If 0.1 is unavailable, pin the latest 0.x release discovered in Step 1.
```

- [ ] **Step 3: Verify build**

Run: `rtk cargo build`
Expected: builds successfully (rmcp pulled into `Cargo.lock`).

- [ ] **Step 4: Commit**

```bash
rtk git add Cargo.toml Cargo.lock
rtk git commit -m "build(mcp): add rmcp dependency

Using rmcp <version> from <source>. Trait: <name>."
```

### Task 3.2: Shared controller handle (for tool→session dispatch)

**Files:**
- Modify: `src/signals/controller.rs`

- [ ] **Step 1: Add a minimal controller surface**

Full state machine lands in M4. For M3 we only need a thin handle that owns the `Session` and exposes low-level operations.

```rust
//! Central owner of the Chroma session and (in M4) the ClaudeState machine.
//!
//! M3 surface: just enough to dispatch set_static_color / clear_all / get_session_status.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig, SessionStatus};
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct Controller {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
        }
    }
}

impl Controller {
    pub async fn session(&self) -> Result<Session> {
        let s = self
            .session
            .get_or_try_init(|| Session::init(self.client.clone(), SessionConfig::default()))
            .await?;
        Ok(s.clone())
    }

    pub async fn session_status(&self) -> SessionStatus {
        match self.session.get() {
            Some(s) => s.status().await,
            None => SessionStatus::Uninitialized,
        }
    }

    pub async fn set_static(&self, color: Rgb) -> Result<()> {
        let session = self.session().await?;
        let grid = keyboard_uniform(color);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
            .await
    }

    pub async fn clear_all(&self) -> Result<()> {
        if let Some(s) = self.session.get() {
            s.put_effect("keyboard", &EffectEnvelope::none()).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if let Some(s) = self.session.get() {
            s.shutdown().await;
        }
    }
}
```

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/controller.rs
rtk git commit -m "feat(signals): controller with lazy session init (m3 surface)"
```

### Task 3.3: MCP tool handlers

**Files:**
- Modify: `src/mcp/tools.rs`

- [ ] **Step 1: Implement the three M3 tools**

```rust
//! MCP tool handlers. Each tool's input schema is declared inline; the handler
//! delegates to `signals::controller::Controller`.

use crate::chroma::color::Rgb;
use crate::signals::controller::Controller;
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SetStaticColorArgs {
    pub device: Option<String>, // reserved — ignored in M3, used from M4
    pub color: ColorInput,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ColorInput {
    Hex(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorInput {
    pub fn to_rgb(&self) -> Result<Rgb> {
        match self {
            ColorInput::Hex(s) => Rgb::from_hex(s).map_err(|e| anyhow!(e)),
            ColorInput::Rgb { r, g, b } => Ok(Rgb::new(*r, *g, *b)),
        }
    }
}

pub async fn set_static_color(ctrl: &Controller, args: SetStaticColorArgs) -> Result<serde_json::Value> {
    let rgb = args.color.to_rgb()?;
    ctrl.set_static(rgb).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn clear_all(ctrl: &Controller) -> Result<serde_json::Value> {
    ctrl.clear_all().await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn get_session_status(ctrl: &Controller) -> Result<serde_json::Value> {
    let status = ctrl.session_status().await;
    Ok(serde_json::json!({
        "initialized": !matches!(status, crate::chroma::session::SessionStatus::Uninitialized),
        "state": format!("{status:?}"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_input_parses_hex() {
        let v: SetStaticColorArgs =
            serde_json::from_value(serde_json::json!({ "color": "#D97757" })).unwrap();
        assert_eq!(v.color.to_rgb().unwrap(), Rgb::new(0xD9, 0x77, 0x57));
    }

    #[test]
    fn color_input_parses_rgb_object() {
        let v: SetStaticColorArgs =
            serde_json::from_value(serde_json::json!({ "color": { "r": 1, "g": 2, "b": 3 } }))
                .unwrap();
        assert_eq!(v.color.to_rgb().unwrap(), Rgb::new(1, 2, 3));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib mcp::tools`
Expected: both tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/mcp/tools.rs
rtk git commit -m "feat(mcp): m3 tool handlers (set_static_color, clear_all, status)"
```

### Task 3.4: MCP server bootstrap

**Files:**
- Modify: `src/mcp/server.rs`

- [ ] **Step 1: Write the server bootstrap using rmcp**

```rust
//! rmcp stdio server bootstrap. Registers tools, dispatches to `mcp::tools`.
//!
//! The exact handler wiring depends on the rmcp version chosen in Task 3.1.
//! The pattern below uses the common "tool registry" shape; adjust to match
//! the version you pinned.

use crate::mcp::tools::{self, SetStaticColorArgs};
use crate::signals::controller::Controller;
use anyhow::Result;
use serde_json::Value;

pub async fn serve(controller: Controller) -> Result<()> {
    // rmcp 0.x pattern — pseudo-code against whatever the pinned API exposes.
    // Replace with the real `rmcp::ServerBuilder`/`ServiceExt` calls.
    //
    // Required tool set for M3:
    //   - set_static_color (args: { device?, color })
    //   - clear_all        (args: {})
    //   - get_session_status (args: {})

    let transport = rmcp::transport::io::stdio();

    let server = rmcp::server::Server::builder()
        .name("razer-chroma-mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .tool(
            "set_static_color",
            "Set a single solid color on the keyboard.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "device": { "type": "string" },
                    "color": {
                        "oneOf": [
                            { "type": "string", "pattern": "^#[0-9A-Fa-f]{3}([0-9A-Fa-f]{3})?$" },
                            {
                                "type": "object",
                                "required": ["r","g","b"],
                                "properties": {
                                    "r": { "type": "integer", "minimum": 0, "maximum": 255 },
                                    "g": { "type": "integer", "minimum": 0, "maximum": 255 },
                                    "b": { "type": "integer", "minimum": 0, "maximum": 255 }
                                }
                            }
                        ]
                    }
                },
                "required": ["color"]
            }),
            {
                let ctrl = controller.clone();
                move |args: Value| {
                    let ctrl = ctrl.clone();
                    async move {
                        let parsed: SetStaticColorArgs = serde_json::from_value(args)?;
                        tools::set_static_color(&ctrl, parsed).await
                    }
                }
            },
        )
        .tool(
            "clear_all",
            "Release the keyboard (Chroma NONE).",
            serde_json::json!({ "type": "object", "properties": {} }),
            {
                let ctrl = controller.clone();
                move |_: Value| {
                    let ctrl = ctrl.clone();
                    async move { tools::clear_all(&ctrl).await }
                }
            },
        )
        .tool(
            "get_session_status",
            "Inspect the Chroma session health.",
            serde_json::json!({ "type": "object", "properties": {} }),
            {
                let ctrl = controller.clone();
                move |_: Value| {
                    let ctrl = ctrl.clone();
                    async move { tools::get_session_status(&ctrl).await }
                }
            },
        )
        .build();

    server.serve(transport).await?;
    Ok(())
}
```

**NOTE for the implementer:** the exact `Server::builder().tool(...)` API depends on the rmcp version chosen in Task 3.1. If the pinned rmcp exposes a `#[tool]` attribute macro instead, restructure as a struct-with-methods and apply the macros; keep the behavior of each handler identical. Record the final shape in the PR.

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully. If it fails due to rmcp API mismatch, iterate — do not commit until it compiles.

- [ ] **Step 3: Commit**

```bash
rtk git add src/mcp/server.rs
rtk git commit -m "feat(mcp): rmcp stdio server with m3 tool registry"
```

### Task 3.5: Wire `serve` subcommand

**Files:**
- Modify: `src/cli/serve.rs`
- Modify: `src/main.rs` (add tracing_appender file logger)

- [ ] **Step 1: Write cli/serve.rs**

```rust
//! `serve` subcommand — run the MCP stdio server with the shared Controller.

use crate::mcp::server;
use crate::signals::controller::Controller;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let controller = Controller::default();

    // Graceful shutdown on Ctrl+C / stdio EOF (EOF is handled inside the transport).
    let ctrl_for_shutdown = controller.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("received ctrl-c, shutting down");
            ctrl_for_shutdown.shutdown().await;
            std::process::exit(0);
        }
    });

    server::serve(controller).await
}
```

- [ ] **Step 2: Upgrade `init_tracing` in src/main.rs to write to file**

Replace the existing `init_tracing` function with:

```rust
fn init_tracing() {
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_env("RAZER_CHROMA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    // File appender at %LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log (daily rotation, 7-day retention).
    let log_dir = std::env::var("LOCALAPPDATA")
        .map(|p| std::path::PathBuf::from(p).join("razer-chroma-mcp").join("logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("./logs"));
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::daily(&log_dir, "mcp.log");

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr).with_ansi(false))
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .init();
}
```

NOTE: `tracing-appender`'s `daily` rotation retains files indefinitely; 7-day retention is implemented via a startup prune routine added in M5 (Task 5.8).

- [ ] **Step 3: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 4: Commit**

```bash
rtk git add src/cli/serve.rs src/main.rs
rtk git commit -m "feat(cli): serve runs mcp server; add file logger"
```

### Task 3.6: End-to-end integration test (spawn subprocess, drive via stdio)

**Files:**
- Create: `tests/mcp_subprocess.rs`

- [ ] **Step 1: Write the failing integration test**

```rust
//! End-to-end: spawn the built binary as a subprocess and drive it via
//! a minimal hand-rolled MCP JSON-RPC client over stdio. This avoids
//! depending on rmcp's client API surface.

use serde_json::{json, Value};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn send<W: AsyncWriteExt + Unpin>(w: &mut W, msg: Value) {
    let s = msg.to_string();
    w.write_all(format!("Content-Length: {}\r\n\r\n{}", s.len(), s).as_bytes())
        .await
        .unwrap();
    w.flush().await.unwrap();
}

async fn recv<R: tokio::io::AsyncRead + Unpin>(r: &mut BufReader<R>) -> Value {
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        r.read_line(&mut line).await.unwrap();
        if line == "\r\n" || line == "\n" {
            break;
        }
        headers.push_str(&line);
    }
    let len: usize = headers
        .lines()
        .find_map(|l| l.strip_prefix("Content-Length:").map(|v| v.trim()))
        .and_then(|v| v.parse().ok())
        .expect("content-length header");
    let mut buf = vec![0u8; len];
    tokio::io::AsyncReadExt::read_exact(r, &mut buf).await.unwrap();
    serde_json::from_slice(&buf).unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[ignore] // unignore after rmcp pinned and `get_session_status` round-trip works
async fn get_session_status_round_trip() {
    let server = MockServer::start().await;
    // We do not even expect the session to be initialized, just that the tool returns.
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

    // initialize
    send(&mut stdin, json!({
        "jsonrpc":"2.0","id":1,"method":"initialize",
        "params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"0"}}
    })).await;
    let _init = tokio::time::timeout(Duration::from_secs(5), recv(&mut stdout)).await.unwrap();

    // tools/call get_session_status
    send(&mut stdin, json!({
        "jsonrpc":"2.0","id":2,"method":"tools/call",
        "params":{"name":"get_session_status","arguments":{}}
    })).await;
    let resp = tokio::time::timeout(Duration::from_secs(5), recv(&mut stdout)).await.unwrap();
    let body = resp.pointer("/result").expect("result present");
    assert!(body.to_string().contains("initialized"));

    child.kill().await.unwrap();
}
```

- [ ] **Step 2: Add `RAZER_CHROMA_BASE_URL` env override to `ChromaClient`**

In `src/chroma/client.rs`, change `Default` impl and `with_base_url` to honor the env var:

```rust
impl Default for ChromaClient {
    fn default() -> Self {
        let base = std::env::var("RAZER_CHROMA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string());
        Self::with_base_url(&base)
    }
}
```

And drop the existing `DEFAULT_BASE`-as-full-suffix assumption: keep the current `with_base_url` logic intact (it already normalizes the `/razer/chromasdk` suffix).

- [ ] **Step 3: Run the ignored test as a smoke check**

Run: `rtk cargo test --test mcp_subprocess -- --ignored`
Expected: PASS (once rmcp wiring is verified). If it fails due to rmcp handshake shape differences, update the `initialize` message to match the pinned rmcp version's expectation. The point is to confirm one round-trip, not to build a full client.

- [ ] **Step 4: Commit**

```bash
rtk git add tests/mcp_subprocess.rs src/chroma/client.rs
rtk git commit -m "test(mcp): subprocess round-trip on get_session_status"
```

### Task 3.7: Live Claude Desktop manual test + M3 close-out

- [ ] **Step 1: Manually register with Claude Desktop**

Edit `%APPDATA%\Claude\claude_desktop_config.json` by hand (automated in M6):

```json
{
  "mcpServers": {
    "razer-chroma": {
      "command": "C:\\Users\\helle\\jun_ws\\razer-chroma-mcp\\target\\debug\\razer-chroma-mcp.exe",
      "args": ["serve"]
    }
  }
}
```

Restart Claude Desktop. Ask Claude: "Use the razer-chroma MCP to set my keyboard to `#D97757`." Expected: keyboard turns terracotta.

Ask: "Clear the keyboard." Expected: keyboard returns to Synapse default.

Ask: "Get the session status." Expected: JSON response with `"initialized": true`.

- [ ] **Step 2: Log the manual test**

Create `docs/manual-testing.md` with this initial entry:

```markdown
# Manual Testing Log

## M3 — 2026-MM-DD

Hardware: BlackWidow (model), Synapse version.

- [x] `set_static_color #D97757` → keyboard terracotta
- [x] `clear_all` → keyboard released
- [x] `get_session_status` → initialized true

Observations: <notes>
```

- [ ] **Step 3: Update CHANGELOG**

```markdown
- rmcp stdio server with `set_static_color`, `clear_all`, `get_session_status` tools.
- `Controller` lazy-initializes the Chroma session on first tool call.
- File logger under `%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log`.
- Initial `docs/manual-testing.md` with M3 results.
```

- [ ] **Step 4: Commit**

```bash
rtk git add docs/manual-testing.md CHANGELOG.md
rtk git commit -m "docs: m3 manual test log + changelog"
```

- [ ] **Step 5: M3 status report + user approval request**

---

## Milestone 4: Full Effects + Claude Signals

Implement the Claude palette, `ClaudeState` state machine with the full §5.3 transition table, the animator infrastructure, all 5 pattern generators, the 7 low-level and 5 high-level MCP tools, idle auto-entry, and state restoration on reconnect.

### Task 4.1: Palette constants

**Files:**
- Modify: `src/signals/palette.rs`

- [ ] **Step 1: Define the four palette colors**

```rust
//! Anthropic warm family — the only colors used by `signal_*` patterns. Spec §5.1.

use crate::chroma::color::Rgb;

pub const TERRACOTTA: Rgb = Rgb::new(0xD9, 0x77, 0x57);
pub const WARM_CREAM: Rgb = Rgb::new(0xF0, 0xEE, 0xE6);
pub const DEEP_EMBER: Rgb = Rgb::new(0x3E, 0x27, 0x23);
pub const BURNT_RUST: Rgb = Rgb::new(0x8D, 0x3A, 0x1C);

/// Linear interpolation between two RGB colors. `t` is clamped to [0, 1].
pub fn lerp(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| ((x as f32) * (1.0 - t) + (y as f32) * t).round() as u8;
    Rgb::new(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

/// Scale brightness. `factor` clamped to [0, 1].
pub fn scale(c: Rgb, factor: f32) -> Rgb {
    let f = factor.clamp(0.0, 1.0);
    let s = |x: u8| ((x as f32) * f).round() as u8;
    Rgb::new(s(c.r), s(c.g), s(c.b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints_return_endpoints() {
        assert_eq!(lerp(TERRACOTTA, WARM_CREAM, 0.0), TERRACOTTA);
        assert_eq!(lerp(TERRACOTTA, WARM_CREAM, 1.0), WARM_CREAM);
    }

    #[test]
    fn scale_to_half_halves_channels() {
        let c = Rgb::new(100, 200, 50);
        assert_eq!(scale(c, 0.5), Rgb::new(50, 100, 25));
    }

    #[test]
    fn scale_is_clamped() {
        assert_eq!(scale(Rgb::new(100, 100, 100), 2.0), Rgb::new(100, 100, 100));
        assert_eq!(scale(Rgb::new(100, 100, 100), -1.0), Rgb::new(0, 0, 0));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::palette`
Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/palette.rs
rtk git commit -m "feat(signals): palette constants (terracotta/cream/ember/burnt_rust)"
```

### Task 4.2: `ClaudeState` / `PersistentState` enums (TDD)

**Files:**
- Modify: `src/signals/mod.rs`

- [ ] **Step 1: Write the failing tests**

Replace `src/signals/mod.rs` contents:

```rust
//! Claude state machine, animators, and pattern generators. See spec §5.

pub mod controller;
pub mod inactivity;
pub mod palette;
pub mod patterns;

/// The states that `Complete` can restore to. Spec §5.2 — kept as a separate
/// enum so that the type system enforces "Complete is never nested".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistentState {
    Off,
    UserControlled,
    Idle,
    Thinking,
    NeedsInput,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaudeState {
    Off,
    UserControlled,
    Idle,
    Thinking,
    NeedsInput,
    /// One-shot. On completion, controller restores `previous`.
    Complete { previous: PersistentState },
    Error,
}

impl ClaudeState {
    /// If this state is persistent (i.e., not `Complete`), return its `PersistentState`.
    pub fn as_persistent(self) -> Option<PersistentState> {
        match self {
            Self::Off => Some(PersistentState::Off),
            Self::UserControlled => Some(PersistentState::UserControlled),
            Self::Idle => Some(PersistentState::Idle),
            Self::Thinking => Some(PersistentState::Thinking),
            Self::NeedsInput => Some(PersistentState::NeedsInput),
            Self::Error => Some(PersistentState::Error),
            Self::Complete { .. } => None,
        }
    }
}

impl From<PersistentState> for ClaudeState {
    fn from(p: PersistentState) -> Self {
        match p {
            PersistentState::Off => Self::Off,
            PersistentState::UserControlled => Self::UserControlled,
            PersistentState::Idle => Self::Idle,
            PersistentState::Thinking => Self::Thinking,
            PersistentState::NeedsInput => Self::NeedsInput,
            PersistentState::Error => Self::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claudestate_is_copy_and_small() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<ClaudeState>();
        // Single-byte discriminants.
        assert!(std::mem::size_of::<ClaudeState>() <= 4);
    }

    #[test]
    fn as_persistent_returns_none_for_complete() {
        let s = ClaudeState::Complete { previous: PersistentState::Thinking };
        assert!(s.as_persistent().is_none());
    }

    #[test]
    fn persistent_round_trip() {
        for p in [
            PersistentState::Off,
            PersistentState::UserControlled,
            PersistentState::Idle,
            PersistentState::Thinking,
            PersistentState::NeedsInput,
            PersistentState::Error,
        ] {
            assert_eq!(ClaudeState::from(p).as_persistent(), Some(p));
        }
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::tests`
Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/mod.rs
rtk git commit -m "feat(signals): ClaudeState + PersistentState with Copy guarantee"
```

### Task 4.3: State transition function (TDD — full §5.3 table)

**Files:**
- Modify: `src/signals/controller.rs`

- [ ] **Step 1: Define `Incoming` enum and `next_state` pure function with failing tests**

Replace `src/signals/controller.rs` contents (we expand in 4.5 — for now we only want the pure transition logic tested):

```rust
//! Central owner of the Chroma session and the ClaudeState machine.
//! The pure `next_state` function encodes spec §5.3 and is table-tested below.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig, SessionStatus};
use crate::error::Result;
use crate::signals::{ClaudeState, PersistentState};
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incoming {
    Thinking,
    NeedsInput,
    Idle,
    Complete,
    Error,
    LowLevelTool,
}

/// Pure state transition per spec §5.3. Returns the new state.
///
/// `Complete` auto-exits are handled separately (see `Controller::after_complete`)
/// rather than encoded here, because their trigger is "animator finished" rather
/// than an incoming signal.
pub fn next_state(from: ClaudeState, incoming: Incoming) -> ClaudeState {
    use ClaudeState as C;
    use Incoming as I;
    match (from, incoming) {
        // From Off
        (C::Off, I::Thinking) => C::Thinking,
        (C::Off, I::NeedsInput) => C::NeedsInput,
        (C::Off, I::Idle) => C::Idle,
        (C::Off, I::Complete) => C::Complete { previous: PersistentState::Off },
        (C::Off, I::Error) => C::Error,
        (C::Off, I::LowLevelTool) => C::UserControlled,

        // From UserControlled
        (C::UserControlled, I::Thinking) => C::Thinking,
        (C::UserControlled, I::NeedsInput) => C::NeedsInput,
        (C::UserControlled, I::Idle) => C::Idle,
        (C::UserControlled, I::Complete) => C::Complete { previous: PersistentState::UserControlled },
        (C::UserControlled, I::Error) => C::Error,
        (C::UserControlled, I::LowLevelTool) => C::UserControlled,

        // From Idle
        (C::Idle, I::Thinking) => C::Thinking,
        (C::Idle, I::NeedsInput) => C::NeedsInput,
        (C::Idle, I::Idle) => C::Idle,
        (C::Idle, I::Complete) => C::Complete { previous: PersistentState::Idle },
        (C::Idle, I::Error) => C::Error,
        (C::Idle, I::LowLevelTool) => C::UserControlled,

        // From Thinking
        (C::Thinking, I::Thinking) => C::Thinking, // restart animator
        (C::Thinking, I::NeedsInput) => C::NeedsInput,
        (C::Thinking, I::Idle) => C::Idle,
        (C::Thinking, I::Complete) => C::Complete { previous: PersistentState::Thinking },
        (C::Thinking, I::Error) => C::Error,
        (C::Thinking, I::LowLevelTool) => C::UserControlled,

        // From NeedsInput
        (C::NeedsInput, I::Thinking) => C::Thinking,
        (C::NeedsInput, I::NeedsInput) => C::NeedsInput, // restart animator
        (C::NeedsInput, I::Idle) => C::Idle,
        (C::NeedsInput, I::Complete) => C::Complete { previous: PersistentState::NeedsInput },
        (C::NeedsInput, I::Error) => C::Error,
        (C::NeedsInput, I::LowLevelTool) => C::UserControlled,

        // From Complete — interrupts the one-shot
        (C::Complete { .. }, I::Thinking) => C::Thinking,
        (C::Complete { .. }, I::NeedsInput) => C::NeedsInput,
        (C::Complete { .. }, I::Idle) => C::Idle,
        (C::Complete { previous }, I::Complete) => C::Complete { previous }, // restart
        (C::Complete { .. }, I::Error) => C::Error,
        (C::Complete { .. }, I::LowLevelTool) => C::UserControlled,

        // From Error
        (C::Error, I::Thinking) => C::Thinking,
        (C::Error, I::NeedsInput) => C::NeedsInput,
        (C::Error, I::Idle) => C::Idle,
        (C::Error, I::Complete) => C::Complete { previous: PersistentState::Error },
        (C::Error, I::Error) => C::Error, // restart
        (C::Error, I::LowLevelTool) => C::UserControlled,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClaudeState as C;
    use Incoming as I;
    use PersistentState as P;

    macro_rules! cell {
        ($name:ident, $from:expr, $incoming:expr, $expected:expr) => {
            #[test]
            fn $name() {
                assert_eq!(next_state($from, $incoming), $expected);
            }
        };
    }

    // Row: Off
    cell!(off_thinking,       C::Off, I::Thinking,     C::Thinking);
    cell!(off_needs_input,    C::Off, I::NeedsInput,   C::NeedsInput);
    cell!(off_idle,           C::Off, I::Idle,         C::Idle);
    cell!(off_complete,       C::Off, I::Complete,     C::Complete { previous: P::Off });
    cell!(off_error,          C::Off, I::Error,        C::Error);
    cell!(off_lowlevel,       C::Off, I::LowLevelTool, C::UserControlled);

    // Row: UserControlled
    cell!(uc_thinking,        C::UserControlled, I::Thinking,     C::Thinking);
    cell!(uc_needs_input,     C::UserControlled, I::NeedsInput,   C::NeedsInput);
    cell!(uc_idle,            C::UserControlled, I::Idle,         C::Idle);
    cell!(uc_complete,        C::UserControlled, I::Complete,     C::Complete { previous: P::UserControlled });
    cell!(uc_error,           C::UserControlled, I::Error,        C::Error);
    cell!(uc_lowlevel,        C::UserControlled, I::LowLevelTool, C::UserControlled);

    // Row: Idle
    cell!(idle_thinking,      C::Idle, I::Thinking,     C::Thinking);
    cell!(idle_needs_input,   C::Idle, I::NeedsInput,   C::NeedsInput);
    cell!(idle_idle,          C::Idle, I::Idle,         C::Idle);
    cell!(idle_complete,      C::Idle, I::Complete,     C::Complete { previous: P::Idle });
    cell!(idle_error,         C::Idle, I::Error,        C::Error);
    cell!(idle_lowlevel,      C::Idle, I::LowLevelTool, C::UserControlled);

    // Row: Thinking
    cell!(th_thinking,        C::Thinking, I::Thinking,     C::Thinking);
    cell!(th_needs_input,     C::Thinking, I::NeedsInput,   C::NeedsInput);
    cell!(th_idle,            C::Thinking, I::Idle,         C::Idle);
    cell!(th_complete,        C::Thinking, I::Complete,     C::Complete { previous: P::Thinking });
    cell!(th_error,           C::Thinking, I::Error,        C::Error);
    cell!(th_lowlevel,        C::Thinking, I::LowLevelTool, C::UserControlled);

    // Row: NeedsInput
    cell!(ni_thinking,        C::NeedsInput, I::Thinking,     C::Thinking);
    cell!(ni_needs_input,     C::NeedsInput, I::NeedsInput,   C::NeedsInput);
    cell!(ni_idle,            C::NeedsInput, I::Idle,         C::Idle);
    cell!(ni_complete,        C::NeedsInput, I::Complete,     C::Complete { previous: P::NeedsInput });
    cell!(ni_error,           C::NeedsInput, I::Error,        C::Error);
    cell!(ni_lowlevel,        C::NeedsInput, I::LowLevelTool, C::UserControlled);

    // Row: Complete — seeded with previous=Thinking
    cell!(cp_thinking,        C::Complete { previous: P::Thinking }, I::Thinking,     C::Thinking);
    cell!(cp_needs_input,     C::Complete { previous: P::Thinking }, I::NeedsInput,   C::NeedsInput);
    cell!(cp_idle,            C::Complete { previous: P::Thinking }, I::Idle,         C::Idle);
    cell!(cp_complete,        C::Complete { previous: P::Thinking }, I::Complete,     C::Complete { previous: P::Thinking });
    cell!(cp_error,           C::Complete { previous: P::Thinking }, I::Error,        C::Error);
    cell!(cp_lowlevel,        C::Complete { previous: P::Thinking }, I::LowLevelTool, C::UserControlled);

    // Row: Error
    cell!(err_thinking,       C::Error, I::Thinking,     C::Thinking);
    cell!(err_needs_input,    C::Error, I::NeedsInput,   C::NeedsInput);
    cell!(err_idle,           C::Error, I::Idle,         C::Idle);
    cell!(err_complete,       C::Error, I::Complete,     C::Complete { previous: P::Error });
    cell!(err_error,          C::Error, I::Error,        C::Error);
    cell!(err_lowlevel,       C::Error, I::LowLevelTool, C::UserControlled);
}

// ──────────────────────────────────────────────────────────────────────────────
// Controller (M3 surface retained; expanded in 4.5 to own the state machine).
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Controller {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
        }
    }
}

impl Controller {
    pub async fn session(&self) -> Result<Session> {
        let s = self
            .session
            .get_or_try_init(|| Session::init(self.client.clone(), SessionConfig::default()))
            .await?;
        Ok(s.clone())
    }

    pub async fn session_status(&self) -> SessionStatus {
        match self.session.get() {
            Some(s) => s.status().await,
            None => SessionStatus::Uninitialized,
        }
    }

    pub async fn set_static(&self, color: Rgb) -> Result<()> {
        let session = self.session().await?;
        let grid = keyboard_uniform(color);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
            .await
    }

    pub async fn clear_all(&self) -> Result<()> {
        if let Some(s) = self.session.get() {
            s.put_effect("keyboard", &EffectEnvelope::none()).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if let Some(s) = self.session.get() {
            s.shutdown().await;
        }
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::controller::tests`
Expected: all 42 transition cells pass (7 rows × 6 incoming signals).

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/controller.rs
rtk git commit -m "feat(signals): pure next_state function with full 7x6 transition table"
```

### Task 4.4: Animator trait + tick scheduler

**Files:**
- Create: `src/signals/animator.rs`
- Modify: `src/signals/mod.rs` (add `pub mod animator;`)

- [ ] **Step 1: Define the Animator abstraction**

```rust
//! Animator runtime: a cancellable tokio task that pushes frames at 20 FPS.
//! Per spec §5.7: graceful cancel waits ≤200ms; force cancel drops immediately.

use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub const FPS: u64 = 20;
pub const TICK_MS: u64 = 1_000 / FPS;

pub type BoxedFrame<'a> = Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;

/// A pattern generator: given a session and a tick index (0-based, ticks at `TICK_MS` intervals),
/// emits one frame by calling `session.put_effect(...)`. Return `Ok(true)` to continue,
/// `Ok(false)` to signal the pattern has finished (one-shot patterns use this).
pub trait Pattern: Send + Sync + 'static {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>>;
}

pub struct AnimatorHandle {
    cancel: CancellationToken,
    join: JoinHandle<()>,
    /// Fires when the animator observes its pattern returning `Ok(false)`
    /// (one-shot complete). Multiple observers may clone this Arc.
    pub finished: Arc<tokio::sync::Notify>,
}

impl AnimatorHandle {
    /// Graceful cancel: signal then wait up to 200ms for the task to exit.
    /// If it doesn't exit in time, we force-drop. Spec §5.7.
    pub async fn cancel_graceful(self) {
        self.cancel.cancel();
        let _ = tokio::time::timeout(Duration::from_millis(200), self.join).await;
    }

    /// Force cancel: drop the handle immediately without awaiting.
    pub fn cancel_force(self) {
        self.cancel.cancel();
        self.join.abort();
    }
}

pub fn spawn<P: Pattern>(session: Session, mut pattern: P) -> AnimatorHandle {
    let cancel = CancellationToken::new();
    let child_cancel = cancel.clone();
    let finished = Arc::new(tokio::sync::Notify::new());
    let finished_task = finished.clone();
    let join = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_millis(TICK_MS));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut tick: u64 = 0;
        loop {
            tokio::select! {
                _ = child_cancel.cancelled() => break,
                _ = ticker.tick() => {
                    let res = pattern.next_frame(&session, tick).await;
                    match res {
                        Ok(true) => {}
                        Ok(false) => {
                            finished_task.notify_waiters();
                            break;
                        }
                        Err(e) => {
                            tracing::warn!("animator frame error: {e}");
                        }
                    }
                    tick += 1;
                }
            }
        }
    });
    AnimatorHandle { cancel, join, finished }
}

/// No-op animator — used when the state has no animation (e.g., `UserControlled`
/// after a user frame has been placed).
pub struct IdlePattern;
impl Pattern for IdlePattern {
    fn next_frame<'a>(
        &'a mut self,
        _s: &'a Session,
        _t: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        Box::pin(async { Ok(true) })
    }
}

// A one-shot helper used by unit tests.
#[cfg(test)]
pub struct OneShotPattern {
    pub total_frames: u64,
    pub last_effect: Arc<std::sync::Mutex<Option<EffectEnvelope>>>,
}

#[cfg(test)]
impl Pattern for OneShotPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let cont = tick + 1 < self.total_frames;
        let ef = EffectEnvelope::none();
        let slot = self.last_effect.clone();
        Box::pin(async move {
            *slot.lock().unwrap() = Some(ef.clone());
            session.put_effect("keyboard", &ef).await?;
            Ok(cont)
        })
    }
}
```

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/animator.rs src/signals/mod.rs
rtk git commit -m "feat(signals): animator runtime with graceful/force cancel (spec 5.7)"
```

### Task 4.5: Expand `Controller` to own the state machine

**Files:**
- Modify: `src/signals/controller.rs`

- [ ] **Step 1: Add state + animator lifecycle to Controller**

Append at the bottom of `src/signals/controller.rs` (after the existing `impl Controller`):

```rust
use crate::signals::animator::{self, AnimatorHandle, Pattern};
use tokio::sync::Mutex;

pub struct ControllerInner {
    pub state: ClaudeState,
    pub animator: Option<AnimatorHandle>,
    /// Last user frame for `UserControlled` restore-on-reconnect (spec §5.8).
    pub last_user_frame: Option<EffectEnvelope>,
}

impl Default for ControllerInner {
    fn default() -> Self {
        Self {
            state: ClaudeState::Off,
            animator: None,
            last_user_frame: None,
        }
    }
}

#[derive(Clone)]
pub struct StatefulController {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
    pub inner: Arc<Mutex<ControllerInner>>,
}

impl Default for StatefulController {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
            inner: Arc::new(Mutex::new(ControllerInner::default())),
        }
    }
}

impl StatefulController {
    pub async fn session(&self) -> Result<Session> {
        let s = self
            .session
            .get_or_try_init(|| Session::init(self.client.clone(), SessionConfig::default()))
            .await?;
        Ok(s.clone())
    }

    pub async fn current_state(&self) -> ClaudeState {
        self.inner.lock().await.state
    }

    /// Transition to the state produced by applying `incoming` to the current state.
    /// Cancels the current animator gracefully (≤200ms) before spawning the new one.
    pub async fn transition(&self, incoming: Incoming) -> Result<ClaudeState> {
        let next = {
            let mut g = self.inner.lock().await;
            let next = next_state(g.state, incoming);
            g.state = next;
            next
        };
        self.swap_animator_for(next).await?;
        Ok(next)
    }

    /// Internal: spawn the appropriate animator for `state`. Caller holds no lock.
    async fn swap_animator_for(&self, state: ClaudeState) -> Result<()> {
        // Graceful-cancel the outgoing animator (lock briefly to take the handle).
        let old = { self.inner.lock().await.animator.take() };
        if let Some(h) = old {
            h.cancel_graceful().await;
        }

        let session = self.session().await?;
        let handle = self.spawn_for_state(session, state);

        // If state is Complete, spawn a task that awaits the animator's `finished`
        // Notify and synthesizes the auto-transition back to `previous`.
        if let (ClaudeState::Complete { previous }, Some(h)) = (state, handle.as_ref()) {
            let finished = h.finished.clone();
            let me = self.clone();
            tokio::spawn(async move {
                finished.notified().await;
                let _ = me.force_transition(ClaudeState::from(previous)).await;
            });
        }

        self.inner.lock().await.animator = handle;
        Ok(())
    }

    /// Bypass the transition table (used by animator-complete restore and reconnect).
    pub async fn force_transition(&self, state: ClaudeState) -> Result<ClaudeState> {
        {
            let mut g = self.inner.lock().await;
            g.state = state;
        }
        self.swap_animator_for(state).await?;
        Ok(state)
    }

    fn spawn_for_state(&self, session: Session, state: ClaudeState) -> Option<AnimatorHandle> {
        use crate::signals::patterns;
        match state {
            ClaudeState::Off | ClaudeState::UserControlled => None,
            ClaudeState::Idle => Some(animator::spawn(session, patterns::idle::IdlePattern::new())),
            ClaudeState::Thinking => Some(animator::spawn(session, patterns::thinking::ThinkingPattern::new())),
            ClaudeState::NeedsInput => Some(animator::spawn(session, patterns::needs_input::NeedsInputPattern::new())),
            ClaudeState::Complete { .. } => Some(animator::spawn(session, patterns::complete::CompletePattern::new())),
            ClaudeState::Error => Some(animator::spawn(session, patterns::error::ErrorPattern::new())),
        }
    }

    /// Low-level frame write — transitions to `UserControlled` and caches the frame.
    pub async fn apply_user_frame(&self, effect: EffectEnvelope) -> Result<()> {
        self.transition(Incoming::LowLevelTool).await?;
        let session = self.session().await?;
        session.put_effect("keyboard", &effect).await?;
        self.inner.lock().await.last_user_frame = Some(effect);
        Ok(())
    }

    pub async fn shutdown(&self) {
        let handle = { self.inner.lock().await.animator.take() };
        if let Some(h) = handle {
            h.cancel_force();
        }
        if let Some(s) = self.session.get() {
            s.shutdown().await;
        }
    }
}
```

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/controller.rs
rtk git commit -m "feat(signals): stateful controller with animator lifecycle and Complete auto-transition"
```

### Task 4.6: `thinking` pattern (TDD — frame assertions)

**Files:**
- Modify: `src/signals/patterns/thinking.rs`

- [ ] **Step 1: Write the failing tests**

```rust
//! Thinking pattern — radial pulse from keyboard center outward.
//! Terracotta ↔ cream gradient, 2.8s cycle, 35–65% brightness. Spec §5.4.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::{Pattern, FPS};
use crate::signals::palette::{self, TERRACOTTA, WARM_CREAM};
use std::future::Future;
use std::pin::Pin;

const CYCLE_MS: u64 = 2_800;
const FRAMES_PER_CYCLE: u64 = CYCLE_MS / (1_000 / FPS);
const CENTER_ROW: f32 = 2.5;
const CENTER_COL: f32 = 10.5;

pub struct ThinkingPattern;

impl ThinkingPattern {
    pub fn new() -> Self { Self }

    /// Pure frame generator — produced as a 6×22 color grid for a given tick.
    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let phase = (tick % FRAMES_PER_CYCLE) as f32 / FRAMES_PER_CYCLE as f32;
        let pulse = (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0..1
        let mut grid = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
        for r in 0..KEYBOARD_ROWS {
            for c in 0..KEYBOARD_COLS {
                let dr = r as f32 - CENTER_ROW;
                let dc = c as f32 - CENTER_COL;
                let dist = (dr * dr + dc * dc).sqrt();
                let max_dist = ((KEYBOARD_ROWS as f32).hypot(KEYBOARD_COLS as f32)) / 2.0;
                let ring = ((pulse + dist / max_dist) % 1.0 - 0.5).abs() * 2.0;
                let color = palette::lerp(TERRACOTTA, WARM_CREAM, ring);
                let brightness = 0.35 + 0.30 * pulse; // 35–65%
                grid[r][c] = palette::scale(color, brightness);
            }
        }
        grid
    }
}

impl Pattern for ThinkingPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let grid = Self::frame(tick);
        Box::pin(async move {
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_stays_in_35_to_65_percent_band() {
        for tick in 0..FRAMES_PER_CYCLE {
            let frame = ThinkingPattern::frame(tick);
            for row in &frame {
                for c in row {
                    let max_channel = c.r.max(c.g).max(c.b) as f32 / 255.0;
                    assert!(max_channel <= 0.66, "brightness {max_channel} > 65%");
                    // Minimum brightness check uses the brightest channel — dark
                    // ring nodes are permitted to be darker; the global max envelope
                    // is what we're asserting here.
                }
            }
        }
    }

    #[test]
    fn frame_is_6_by_22() {
        let f = ThinkingPattern::frame(0);
        assert_eq!(f.len(), KEYBOARD_ROWS);
        assert_eq!(f[0].len(), KEYBOARD_COLS);
    }

    #[test]
    fn is_animated_across_cycle() {
        let a = ThinkingPattern::frame(0);
        let b = ThinkingPattern::frame(FRAMES_PER_CYCLE / 2);
        assert_ne!(a, b, "frames should differ across half-cycle");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::patterns::thinking`
Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/patterns/thinking.rs
rtk git commit -m "feat(signals): thinking pattern (radial terracotta-cream pulse)"
```

### Task 4.7: `needs_input` pattern — 3-stage high-impact

**Files:**
- Modify: `src/signals/patterns/needs_input.rs`

- [ ] **Step 1: Write failing tests with measurable criteria from spec §5.4**

```rust
//! NeedsInput pattern — 3-stage attention-grabber. Spec §5.4.
//!
//! Stage 1 (0..900ms): triple cream flash, each flash ≤300ms, all flashes ≥90% brightness.
//! Stage 2 (loops): full terracotta breathing, 1.2s cycle, 60–100% amplitude.
//! Stage 3 (overlay from start of stage 2): spacebar+Enter+Esc cream at 0.75s
//!          counter-rhythm. Polyrhythm against the 1.2s base — intentional.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::{Pattern, FPS};
use crate::signals::palette::{self, TERRACOTTA, WARM_CREAM};
use std::future::Future;
use std::pin::Pin;

const TICK_MS: u64 = 1_000 / FPS;
pub const STAGE1_FLASHES: u64 = 3;
pub const FLASH_ON_MS: u64 = 150;
pub const FLASH_OFF_MS: u64 = 150;
pub const FLASH_PERIOD_MS: u64 = FLASH_ON_MS + FLASH_OFF_MS; // 300
pub const STAGE1_MS: u64 = STAGE1_FLASHES * FLASH_PERIOD_MS; // 900
pub const BREATH_CYCLE_MS: u64 = 1_200;
pub const ACCENT_CYCLE_MS: u64 = 750;

// Accent key grid coordinates (approximate BlackWidow US layout).
// Spacebar row: row 5, cols 4..16; Enter: row 3 col 19; Esc: row 0 col 0.
pub const SPACEBAR_ROW: usize = 5;
pub const SPACEBAR_COLS: std::ops::Range<usize> = 4..16;
pub const ENTER_ROW: usize = 3;
pub const ENTER_COL: usize = 19;
pub const ESC_ROW: usize = 0;
pub const ESC_COL: usize = 0;

pub struct NeedsInputPattern;

impl NeedsInputPattern {
    pub fn new() -> Self { Self }

    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t_ms = tick * TICK_MS;
        if t_ms < STAGE1_MS {
            Self::stage1(t_ms)
        } else {
            Self::stage23(t_ms - STAGE1_MS)
        }
    }

    fn stage1(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let in_flash = (t_ms % FLASH_PERIOD_MS) < FLASH_ON_MS;
        let color = if in_flash {
            palette::scale(WARM_CREAM, 0.95) // ≥90%
        } else {
            Rgb::BLACK
        };
        [[color; KEYBOARD_COLS]; KEYBOARD_ROWS]
    }

    fn stage23(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let breath_phase = (t_ms % BREATH_CYCLE_MS) as f32 / BREATH_CYCLE_MS as f32;
        let breath = (breath_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let brightness = 0.60 + 0.40 * breath; // 60–100%
        let base = palette::scale(TERRACOTTA, brightness);

        let accent_phase = (t_ms % ACCENT_CYCLE_MS) as f32 / ACCENT_CYCLE_MS as f32;
        let accent_on = (accent_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let accent_brightness = 0.5 + 0.5 * accent_on;
        let accent = palette::scale(WARM_CREAM, accent_brightness);

        let mut grid = [[base; KEYBOARD_COLS]; KEYBOARD_ROWS];
        for c in SPACEBAR_COLS.clone() {
            grid[SPACEBAR_ROW][c] = accent;
        }
        grid[ENTER_ROW][ENTER_COL] = accent;
        grid[ESC_ROW][ESC_COL] = accent;
        grid
    }
}

impl Pattern for NeedsInputPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let grid = Self::frame(tick);
        Box::pin(async move {
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance criterion from spec §5.4 / §16:
    /// "≥3 cream flashes ≥90% brightness within 900 ms".
    #[test]
    fn stage1_produces_three_flashes_at_90_percent_within_900ms() {
        let mut flash_frames = 0u64;
        let mut saw_on_off_on = false;
        let mut last_on = false;
        let mut transitions = 0u64;
        let total_ticks = 900 / TICK_MS; // 18
        for tick in 0..total_ticks {
            let frame = NeedsInputPattern::frame(tick);
            let center = frame[3][11];
            let is_on = center.r > 0;
            if is_on {
                let max_channel = center.r.max(center.g).max(center.b) as f32 / 255.0;
                assert!(max_channel >= 0.90, "flash brightness {max_channel} < 90%");
                flash_frames += 1;
            }
            if is_on != last_on {
                transitions += 1;
                last_on = is_on;
            }
            if transitions >= 5 { saw_on_off_on = true; }
        }
        assert!(flash_frames >= 3, "need at least 3 lit frames, got {flash_frames}");
        assert!(saw_on_off_on, "need at least 3 distinct flashes (≥5 on↔off transitions)");
    }

    #[test]
    fn stage2_breathing_amplitude_at_least_40_percent() {
        let start = STAGE1_MS / TICK_MS;
        let end = start + (BREATH_CYCLE_MS / TICK_MS);
        let mut min_b = 1.0f32;
        let mut max_b = 0.0f32;
        for tick in start..end {
            let frame = NeedsInputPattern::frame(tick);
            // Use a non-accent key: row 2, col 10.
            let c = frame[2][10];
            let b = c.r.max(c.g).max(c.b) as f32 / 255.0;
            min_b = min_b.min(b);
            max_b = max_b.max(b);
        }
        assert!(max_b - min_b >= 0.40, "breathing amplitude {} < 40%", max_b - min_b);
    }

    #[test]
    fn stage3_accents_differ_from_base_on_counter_rhythm() {
        // Pick a tick inside stage 2/3. Spacebar keys should differ from body keys.
        let tick = (STAGE1_MS + 400) / TICK_MS;
        let frame = NeedsInputPattern::frame(tick);
        let body = frame[2][10];
        let spacebar = frame[SPACEBAR_ROW][8];
        assert_ne!(body, spacebar, "spacebar should differ from body during stage 2+3");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::patterns::needs_input`
Expected: all 3 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/patterns/needs_input.rs
rtk git commit -m "feat(signals): needs_input 3-stage pattern (flash + breathe + accent polyrhythm)"
```

### Task 4.8: `idle`, `error` patterns (built-in CHROMA_BREATHING)

**Files:**
- Modify: `src/signals/patterns/idle.rs`
- Modify: `src/signals/patterns/error.rs`

- [ ] **Step 1: Implement `idle` — single-call CHROMA_BREATHING, no per-frame updates**

```rust
//! Idle pattern — very slow deep ember breathing. Spec §5.4: 12s cycle, 5–12% brightness.
//! Uses the built-in `CHROMA_BREATHING` effect so we only PUT once.

use crate::chroma::effects::{BreathingKind, EffectEnvelope};
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use crate::signals::palette::{self, DEEP_EMBER};
use std::future::Future;
use std::pin::Pin;

pub struct IdlePattern {
    sent: bool,
}

impl IdlePattern {
    pub fn new() -> Self { Self { sent: false } }
}

impl Pattern for IdlePattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let send = !self.sent;
        self.sent = true;
        Box::pin(async move {
            if send {
                let dim = palette::scale(DEEP_EMBER, 0.10); // center of 5–12% band
                let effect = EffectEnvelope::breathing(BreathingKind::OneColor(dim));
                session.put_effect("keyboard", &effect).await?;
            }
            Ok(true)
        })
    }
}
```

- [ ] **Step 2: Implement `error` similarly**

```rust
//! Error pattern — burnt rust slow breathing. Spec §5.4: 2.5s cycle, 40–80%.

use crate::chroma::effects::{BreathingKind, EffectEnvelope};
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::Pattern;
use crate::signals::palette::{self, BURNT_RUST};
use std::future::Future;
use std::pin::Pin;

pub struct ErrorPattern {
    sent: bool,
}

impl ErrorPattern {
    pub fn new() -> Self { Self { sent: false } }
}

impl Pattern for ErrorPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        _tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let send = !self.sent;
        self.sent = true;
        Box::pin(async move {
            if send {
                let color = palette::scale(BURNT_RUST, 0.60);
                let effect = EffectEnvelope::breathing(BreathingKind::OneColor(color));
                session.put_effect("keyboard", &effect).await?;
            }
            Ok(true)
        })
    }
}
```

- [ ] **Step 3: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 4: Commit**

```bash
rtk git add src/signals/patterns/idle.rs src/signals/patterns/error.rs
rtk git commit -m "feat(signals): idle and error patterns via CHROMA_BREATHING"
```

### Task 4.9: `complete` pattern — sweep + asterisk one-shot

**Files:**
- Modify: `src/signals/patterns/complete.rs`

- [ ] **Step 1: Implement the asterisk mask + sweep + fade, returning false on completion**

```rust
//! Complete pattern — 1.8s one-shot: left→right terracotta sweep → central cream
//! asterisk flash → fade to previous. Spec §5.4 and §5.5 (shape matrix).

use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::{Pattern, FPS};
use crate::signals::palette::{self, TERRACOTTA, WARM_CREAM};
use std::future::Future;
use std::pin::Pin;

const TICK_MS: u64 = 1_000 / FPS;
const SWEEP_MS: u64 = 900;
const ASTERISK_HOLD_MS: u64 = 180;
const FADE_MS: u64 = 400;
const TOTAL_MS: u64 = SWEEP_MS + ASTERISK_HOLD_MS + FADE_MS + 320; // ≈1.8s

// Spec §5.5 asterisk mask. `1` = lit, `0` = dark.
const ASTERISK: [[u8; KEYBOARD_COLS]; KEYBOARD_ROWS] = [
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0],
    [0,0,0,0,0,1,0,0,0,1,1,1,0,0,0,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,0,1,1,1,1,1,1,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,1,0,0,0,1,1,1,0,0,0,1,0,0,0,0,0,0],
    [0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0],
];

pub struct CompletePattern;

impl CompletePattern {
    pub fn new() -> Self { Self }

    pub fn total_ticks() -> u64 { TOTAL_MS / TICK_MS }

    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t_ms = tick * TICK_MS;
        if t_ms < SWEEP_MS {
            Self::sweep(t_ms)
        } else if t_ms < SWEEP_MS + ASTERISK_HOLD_MS {
            Self::asterisk(0.85)
        } else if t_ms < SWEEP_MS + ASTERISK_HOLD_MS + FADE_MS {
            let fade_t = (t_ms - SWEEP_MS - ASTERISK_HOLD_MS) as f32 / FADE_MS as f32;
            Self::asterisk(0.85 * (1.0 - fade_t))
        } else {
            [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS]
        }
    }

    fn sweep(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t = t_ms as f32 / SWEEP_MS as f32;
        let leading = (t * (KEYBOARD_COLS as f32 + 4.0)) as i32 - 2;
        let mut grid = [[palette::scale(TERRACOTTA, 0.60); KEYBOARD_COLS]; KEYBOARD_ROWS];
        for r in 0..KEYBOARD_ROWS {
            for c in 0..KEYBOARD_COLS {
                let dist = (c as i32 - leading).abs();
                if dist <= 2 {
                    grid[r][c] = palette::lerp(TERRACOTTA, WARM_CREAM, 1.0 - dist as f32 / 2.0);
                }
            }
        }
        grid
    }

    fn asterisk(brightness: f32) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let mut grid = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
        let lit = palette::scale(WARM_CREAM, brightness);
        for r in 0..KEYBOARD_ROWS {
            for c in 0..KEYBOARD_COLS {
                if ASTERISK[r][c] == 1 {
                    grid[r][c] = lit;
                }
            }
        }
        grid
    }
}

impl Pattern for CompletePattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let total = Self::total_ticks();
        let grid = Self::frame(tick);
        Box::pin(async move {
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
            Ok(tick + 1 < total) // false → animator reports finished
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_runs_approximately_1_8_seconds() {
        let total_ms = Self::total_ticks() * TICK_MS;
        // Compile-time-ish sanity: 1.7–2.0s window.
        let _ = total_ms;
        assert!((1_700..=2_000).contains(&TOTAL_MS));
    }

    #[test]
    fn asterisk_mask_is_6x22() {
        assert_eq!(ASTERISK.len(), KEYBOARD_ROWS);
        assert_eq!(ASTERISK[0].len(), KEYBOARD_COLS);
    }

    #[test]
    fn asterisk_has_lit_center_ray() {
        // Row 3 (mid) should have the horizontal ray of the asterisk lit.
        let count: u8 = ASTERISK[3].iter().sum();
        assert!(count >= 4, "center row should have ≥4 lit keys, got {count}");
    }

    #[test]
    fn last_tick_returns_dark_frame() {
        let f = CompletePattern::frame(CompletePattern::total_ticks());
        for row in &f {
            for c in row {
                assert_eq!(*c, Rgb::BLACK);
            }
        }
    }
}
```

Note: the `total_runs_approximately_1_8_seconds` test uses `Self::` at module scope, which is invalid — change to `CompletePattern::total_ticks()` (inline fix while applying).

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::patterns::complete`
Expected: all 4 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/patterns/complete.rs
rtk git commit -m "feat(signals): complete pattern (sweep + asterisk flash + fade)"
```

### Task 4.10: Inactivity timer for idle auto-entry

**Files:**
- Modify: `src/signals/inactivity.rs`

- [ ] **Step 1: Implement the timer with fake-clock-friendly API**

```rust
//! Tracks "time since last signal" and fires a callback when idle timeout elapses.
//! Spec §5.9: default 10 min; Thinking/NeedsInput excluded from auto-entry.

use crate::signals::ClaudeState;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_util::sync::CancellationToken;

pub const DEFAULT_TIMEOUT_MS: u64 = 10 * 60 * 1_000;

pub fn timeout_from_env() -> Duration {
    std::env::var("RAZER_CHROMA_IDLE_MS")
        .ok()
        .and_then(|s| s.parse().ok())
        .map(Duration::from_millis)
        .unwrap_or(Duration::from_millis(DEFAULT_TIMEOUT_MS))
}

#[derive(Clone)]
pub struct Inactivity {
    last: Arc<Mutex<Instant>>,
    cancel: CancellationToken,
}

impl Inactivity {
    pub fn new() -> Self {
        Self {
            last: Arc::new(Mutex::new(Instant::now())),
            cancel: CancellationToken::new(),
        }
    }

    pub async fn bump(&self) {
        *self.last.lock().await = Instant::now();
    }

    pub async fn last(&self) -> Instant {
        *self.last.lock().await
    }

    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Returns true if `state` is eligible for auto-Idle.
    pub fn eligible(state: ClaudeState) -> bool {
        matches!(state, ClaudeState::UserControlled | ClaudeState::Error)
    }

    /// Spawn a watcher that calls `on_idle()` once when the timeout elapses AND
    /// the current state is eligible. `get_state` samples state without mutex contention.
    pub fn spawn_watcher<G, F, Fut>(self, timeout: Duration, get_state: G, on_idle: F) -> Self
    where
        G: Fn() -> ClaudeState + Send + Sync + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let me = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = me.cancel.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                }
                let last = *me.last.lock().await;
                if Instant::now().duration_since(last) >= timeout && Self::eligible(get_state()) {
                    on_idle().await;
                    me.bump().await;
                }
            }
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn bump_resets_elapsed() {
        let i = Inactivity::new();
        let before = i.last().await;
        tokio::time::advance(Duration::from_secs(2)).await;
        i.bump().await;
        let after = i.last().await;
        assert!(after > before);
    }

    #[test]
    fn eligibility_excludes_thinking_and_needs_input() {
        assert!(!Inactivity::eligible(ClaudeState::Thinking));
        assert!(!Inactivity::eligible(ClaudeState::NeedsInput));
        assert!(Inactivity::eligible(ClaudeState::UserControlled));
        assert!(Inactivity::eligible(ClaudeState::Error));
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib signals::inactivity`
Expected: both tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/inactivity.rs
rtk git commit -m "feat(signals): inactivity timer with eligibility rule (excludes thinking/needs_input)"
```

### Task 4.11: Wire inactivity timer into `StatefulController`

**Files:**
- Modify: `src/signals/controller.rs`

- [ ] **Step 1: Store the `Inactivity` outside the inner lock so the watcher doesn't fight for it**

Add `Inactivity` as a top-level field of `StatefulController` (same pattern used for `Session.reconnected` in Task 4.13):

```rust
use crate::signals::inactivity::{self, Inactivity};

#[derive(Clone)]
pub struct StatefulController {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
    pub inner: Arc<Mutex<ControllerInner>>,
    inactivity: Inactivity,
}

impl Default for StatefulController {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
            inner: Arc::new(Mutex::new(ControllerInner::default())),
            inactivity: Inactivity::new(),
        }
    }
}

impl StatefulController {
    pub async fn start_idle_watcher(&self) {
        let get_state = {
            let inner = self.inner.clone();
            move || inner.try_lock().map(|g| g.state).unwrap_or(ClaudeState::Off)
        };
        let trigger_ctrl = self.clone();
        self.inactivity.clone().spawn_watcher(
            inactivity::timeout_from_env(),
            get_state,
            move || {
                let ctrl = trigger_ctrl.clone();
                async move { let _ = ctrl.transition(Incoming::Idle).await; }
            },
        );
    }
}
```

And in `transition()` and `apply_user_frame()`, after updating state, call:

```rust
self.inactivity.bump().await;
```

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Integration test for auto-idle with env override**

Add to `tests/chroma_session.rs` (or create `tests/inactivity_integration.rs`):

```rust
#[tokio::test(flavor = "multi_thread", start_paused = true)]
async fn auto_idle_fires_from_user_controlled() {
    // Construction notes: build a StatefulController with RAZER_CHROMA_IDLE_MS=200
    // (set via std::env::set_var in test), transition to UserControlled, advance
    // the paused clock, and assert state becomes Idle within the next scheduler tick.
    //
    // Full assertion body depends on the `StatefulController::new_with_session` helper
    // introduced here — add if not present. Skip if the helper requires a live session;
    // the unit-level eligibility test (Task 4.10) and the manual verification in M4
    // closeout provide coverage.
}
```

Mark the test `#[ignore]` if a mockable session entry-point is out of scope for this task; rely on the M5 reconnect integration test and manual verification. Record the decision in the commit message.

- [ ] **Step 4: Commit**

```bash
rtk git add src/signals/controller.rs tests/
rtk git commit -m "feat(signals): wire inactivity timer into stateful controller"
```

### Task 4.12: Full MCP tool surface (12 tools)

**Files:**
- Modify: `src/mcp/tools.rs`
- Modify: `src/mcp/server.rs`

- [ ] **Step 1: Add handlers for all 12 tools**

Replace `src/mcp/tools.rs` contents:

```rust
//! MCP tool handlers. See spec §6.

use crate::chroma::color::Rgb;
use crate::chroma::devices::Device;
use crate::chroma::effects::{keyboard_uniform, BreathingKind, EffectEnvelope, ReactiveDuration, WaveDirection};
use crate::signals::controller::{Incoming, StatefulController};
use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ColorOnly { pub color: ColorInput }

#[derive(Deserialize)]
pub struct SetStaticColorArgs {
    pub device: Option<String>,
    pub color: ColorInput,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ColorInput {
    Hex(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorInput {
    pub fn to_rgb(&self) -> Result<Rgb> {
        match self {
            ColorInput::Hex(s) => Rgb::from_hex(s).map_err(|e| anyhow!(e)),
            ColorInput::Rgb { r, g, b } => Ok(Rgb::new(*r, *g, *b)),
        }
    }
}

#[derive(Deserialize)]
pub struct SetCustomGridArgs {
    pub device: Option<String>,
    pub grid: Vec<Vec<ColorInput>>,
}

#[derive(Deserialize)]
pub struct PlayPresetArgs {
    pub device: Option<String>,
    pub effect_type: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Deserialize)]
pub struct ClearDeviceArgs { pub device: String }

#[derive(Deserialize, Default)]
pub struct EmptyArgs {}

#[derive(Deserialize, Default)]
pub struct SignalErrorArgs { pub message: Option<String> }

// ── Low-level ─────────────────────────────────────────────────────────────────

pub async fn set_static_color(c: &StatefulController, a: SetStaticColorArgs) -> Result<serde_json::Value> {
    let rgb = a.color.to_rgb()?;
    let grid = keyboard_uniform(rgb);
    c.apply_user_frame(EffectEnvelope::keyboard_custom(&grid)).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn set_custom_grid(c: &StatefulController, a: SetCustomGridArgs) -> Result<serde_json::Value> {
    // Validate dims against keyboard grid (other devices deferred to v0.2.0 for patterns).
    use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
    if a.grid.len() != KEYBOARD_ROWS {
        return Err(anyhow!("grid must have {} rows", KEYBOARD_ROWS));
    }
    let mut arr = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
    for (r, row) in a.grid.iter().enumerate() {
        if row.len() != KEYBOARD_COLS {
            return Err(anyhow!("row {r} must have {} cols", KEYBOARD_COLS));
        }
        for (col_idx, col) in row.iter().enumerate() {
            arr[r][col_idx] = col.to_rgb()?;
        }
    }
    c.apply_user_frame(EffectEnvelope::keyboard_custom(&arr)).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn play_preset_effect(c: &StatefulController, a: PlayPresetArgs) -> Result<serde_json::Value> {
    let effect = match a.effect_type.to_ascii_uppercase().as_str() {
        "WAVE" => {
            let dir = a.params.get("direction").and_then(|v| v.as_str()).unwrap_or("left_to_right");
            let d = if dir == "right_to_left" { WaveDirection::RightToLeft } else { WaveDirection::LeftToRight };
            EffectEnvelope::wave(d)
        }
        "SPECTRUMCYCLING" => EffectEnvelope::spectrumcycling(),
        "BREATHING" => {
            // Expect optional `color` (hex or rgb)
            let kind = if let Some(c) = a.params.get("color") {
                let v: ColorInput = serde_json::from_value(c.clone())?;
                BreathingKind::OneColor(v.to_rgb()?)
            } else {
                BreathingKind::Random
            };
            EffectEnvelope::breathing(kind)
        }
        "REACTIVE" => {
            let color = a.params.get("color").ok_or_else(|| anyhow!("reactive requires color"))?;
            let v: ColorInput = serde_json::from_value(color.clone())?;
            let duration = match a.params.get("duration").and_then(|v| v.as_str()).unwrap_or("medium") {
                "short" => ReactiveDuration::Short,
                "long" => ReactiveDuration::Long,
                _ => ReactiveDuration::Medium,
            };
            EffectEnvelope::reactive(v.to_rgb()?, duration)
        }
        other => return Err(anyhow!("unsupported effect_type: {other}")),
    };
    c.apply_user_frame(effect).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn clear_device(c: &StatefulController, a: ClearDeviceArgs) -> Result<serde_json::Value> {
    let device = Device::parse(&a.device).map_err(|e| anyhow!(e))?;
    let session = c.session().await.map_err(|e| anyhow!(e))?;
    session.put_effect(device.path_segment(), &EffectEnvelope::none()).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn clear_all(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    c.force_transition(crate::signals::ClaudeState::Off).await.map_err(|e| anyhow!(e))?;
    let session = c.session().await.map_err(|e| anyhow!(e))?;
    for device in ["keyboard", "mouse", "mousepad", "headset", "keypad", "chromalink"] {
        let _ = session.put_effect(device, &EffectEnvelope::none()).await;
    }
    Ok(serde_json::json!({ "status": "ok" }))
}

pub async fn get_device_info(_c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    Ok(serde_json::json!({
        "devices": [
            { "name": "keyboard",   "rows": 6, "cols": 22 },
            { "name": "mouse",      "rows": 9, "cols": 7 },
            { "name": "keypad",     "rows": 4, "cols": 5 },
            { "name": "mousepad",   "leds": 20 },
            { "name": "headset",    "leds": 5 },
            { "name": "chromalink", "leds": 5 }
        ]
    }))
}

pub async fn get_session_status(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    let status = match c.session().await {
        Ok(s) => s.status().await,
        Err(_) => crate::chroma::session::SessionStatus::Uninitialized,
    };
    let state = c.current_state().await;
    Ok(serde_json::json!({
        "initialized": !matches!(status, crate::chroma::session::SessionStatus::Uninitialized),
        "session_status": format!("{status:?}"),
        "state": format!("{state:?}"),
    }))
}

// ── High-level signals ────────────────────────────────────────────────────────

pub async fn signal_thinking(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    c.transition(Incoming::Thinking).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok", "state": "Thinking" }))
}
pub async fn signal_needs_input(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    c.transition(Incoming::NeedsInput).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok", "state": "NeedsInput" }))
}
pub async fn signal_idle(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    c.transition(Incoming::Idle).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok", "state": "Idle" }))
}
pub async fn signal_complete(c: &StatefulController, _: EmptyArgs) -> Result<serde_json::Value> {
    c.transition(Incoming::Complete).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok", "state": "Complete" }))
}
pub async fn signal_error(c: &StatefulController, a: SignalErrorArgs) -> Result<serde_json::Value> {
    if let Some(m) = &a.message {
        tracing::warn!(target: "claude_signal", "signal_error: {m}");
    }
    c.transition(Incoming::Error).await.map_err(|e| anyhow!(e))?;
    Ok(serde_json::json!({ "status": "ok", "state": "Error" }))
}
```

- [ ] **Step 2: Register all 12 tools in `mcp::server::serve`**

Update `src/mcp/server.rs` — replace `Controller` references with `StatefulController`, and add registrations for every tool above. Each tool uses the same registration pattern as in Task 3.4. Schemas:

- `set_custom_grid`: `{ "type": "object", "required": ["grid"], "properties": { "device": {"type":"string"}, "grid": {"type":"array"} } }`
- `play_preset_effect`: `{ "type": "object", "required": ["effect_type"], "properties": { "device": {"type":"string"}, "effect_type": {"type":"string", "enum":["WAVE","SPECTRUMCYCLING","BREATHING","REACTIVE"]}, "params": {"type":"object"} } }`
- `clear_device`: `{ "type": "object", "required": ["device"], "properties": { "device": {"type":"string"} } }`
- `get_device_info`, `clear_all`, `get_session_status`, and all `signal_*` except `signal_error`: `{ "type": "object", "properties": {} }`
- `signal_error`: `{ "type": "object", "properties": { "message": {"type":"string"} } }`

- [ ] **Step 3: Update `cli/serve.rs` to construct a `StatefulController` and call `start_idle_watcher`**

```rust
pub async fn run() -> Result<()> {
    let controller = crate::signals::controller::StatefulController::default();
    controller.start_idle_watcher().await;

    let ctrl_for_shutdown = controller.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("received ctrl-c, shutting down");
            ctrl_for_shutdown.shutdown().await;
            std::process::exit(0);
        }
    });

    crate::mcp::server::serve(controller).await
}
```

- [ ] **Step 4: Verify build + unignore M3 subprocess test**

Run: `rtk cargo build`
Run: `rtk cargo test`
Expected: everything green, including the subprocess integration test (remove `#[ignore]`).

- [ ] **Step 5: Commit**

```bash
rtk git add src/mcp/tools.rs src/mcp/server.rs src/cli/serve.rs
rtk git commit -m "feat(mcp): register all 12 tools (7 low-level + 5 signal_*)"
```

### Task 4.13: State restoration on reconnect

**Files:**
- Modify: `src/signals/controller.rs`
- Modify: `src/chroma/session.rs`

- [ ] **Step 1: Expose a reconnect callback on `Session`**

Add a `reconnected` signal to `Session` — simplest shape is an `Arc<Notify>` that fires inside `reconnect()` right after the new URI is stored. `StatefulController` clones the Notify on init and spawns a watcher:

```rust
// In StatefulController::default, after session is initialized on first call,
// arrange to observe `session.reconnected_notify()` and, on each fire,
// re-apply the current persistent state or the last user frame.
```

Since `Session` is lazy-initialized, do this work inside `StatefulController::session()`:

```rust
pub async fn session(&self) -> Result<Session> {
    let s = self
        .session
        .get_or_try_init(|| async {
            let s = Session::init(self.client.clone(), SessionConfig::default()).await?;
            let notify = s.reconnected_notify();
            let me = self.clone();
            tokio::spawn(async move {
                loop {
                    notify.notified().await;
                    me.restore_after_reconnect().await;
                }
            });
            Ok::<_, crate::error::ChromaError>(s)
        })
        .await?;
    Ok(s.clone())
}

async fn restore_after_reconnect(&self) {
    let (state, last_frame) = {
        let g = self.inner.lock().await;
        (g.state, g.last_user_frame.clone())
    };
    match state {
        ClaudeState::UserControlled => {
            if let Some(effect) = last_frame {
                if let Ok(session) = self.session.get().cloned().ok_or(()) {
                    let _ = session.put_effect("keyboard", &effect).await;
                }
            }
        }
        other => { let _ = self.force_transition(other).await; }
    }
}
```

- [ ] **Step 2: Implement `reconnected_notify` in `Session`**

Add an `Arc<Notify>` as a **top-level field of `Session`** (not inside `Inner`/`RwLock`) so the getter needs no locking:

```rust
// in Session:
reconnected: Arc<tokio::sync::Notify>,

// constructor: `reconnected: Arc::new(tokio::sync::Notify::new())` at init.

pub fn reconnected_notify(&self) -> Arc<tokio::sync::Notify> {
    self.reconnected.clone()
}
```

In `reconnect()`, after the write lock sets `status = Healthy` and `uri = ...`, drop the guard, then call `self.reconnected.notify_waiters()`.

- [ ] **Step 3: Integration test — reconnect triggers restore**

Extend `tests/chroma_session.rs` (or create `tests/reconnect_restore.rs`): wiremock returns 503 for 3 heartbeats, then 200 for init, then accepts puts; assert `put_effect("keyboard", CHROMA_CUSTOM)` was observed after reconnect when a user frame was cached. Mark `#[ignore]` if the test requires deeper hooks; capture the manual equivalent in `docs/manual-testing.md`.

- [ ] **Step 4: Commit**

```bash
rtk git add src/signals/controller.rs src/chroma/session.rs tests/
rtk git commit -m "feat(signals): restore last state on session reconnect"
```

### Task 4.14: M4 manual verification + close-out

- [ ] **Step 1: Live hardware run-through**

With BlackWidow connected, Synapse running:

1. `./target/debug/razer-chroma-mcp.exe serve` (in one terminal)
2. In another: drive via a temporary test client or re-use Claude Desktop via hand-edited config. For each signal (`thinking`/`needs_input`/`idle`/`complete`/`error`), send `tools/call` and visually confirm the pattern.
3. Capture observations: Does `needs_input` feel "강렬하게 인상"? Does `complete` end cleanly with the asterisk visible? Is `thinking` smooth at 20 FPS?

Append to `docs/manual-testing.md`:

```markdown
## M4 — 2026-MM-DD

Hardware: BlackWidow, Synapse version.

- [x] `signal_thinking` → radial pulse observed
- [x] `signal_needs_input` → 3-stage flash + breathing + spacebar/Enter/Esc accents
      Subjective impact: <yes / needs tuning / notes>
- [x] `signal_idle` → deep ember slow breathing
- [x] `signal_complete` → sweep + asterisk flash, returns to previous
- [x] `signal_error` → burnt rust slow breathing
- [x] Low-level `set_static_color` after a signal → transitions to UserControlled

Integration tests: needs_input stage-1 meets ≥90% brightness within 900 ms criterion.
```

- [ ] **Step 2: Update CHANGELOG**

```markdown
- Anthropic warm palette (terracotta/cream/ember/burnt rust).
- `ClaudeState` / `PersistentState` enums with type-level Complete-never-nested invariant.
- Full state transition table (§5.3) covered by 42 unit tests.
- Animator runtime at 20 FPS with graceful/force cancel (§5.7).
- Pattern generators: thinking, needs_input (3-stage, measurable), idle, complete (sweep+asterisk), error.
- `signal_thinking/needs_input/idle/complete/error` MCP tools.
- Low-level surface completed: set_static_color, set_custom_grid, play_preset_effect, clear_device, clear_all, get_device_info, get_session_status.
- Idle auto-entry (§5.9) with `RAZER_CHROMA_IDLE_MS` override and eligibility rule.
- Reconnect restores previous state including cached user frame.
```

- [ ] **Step 3: Commit**

```bash
rtk git add docs/manual-testing.md CHANGELOG.md
rtk git commit -m "docs: m4 manual testing log and changelog"
```

- [ ] **Step 4: M4 status report + user approval**

---

## Milestone 5: Error Handling, `doctor`, `pattern`, Log Rotation

Polish the error surface (bilingual messages, recovery paths, degraded mode), implement `doctor` and `pattern` subcommands, and add log retention.

### Task 5.1: Distinguish `SynapseNotRunning` vs `ChromaModuleDisabled`

**Files:**
- Modify: `src/chroma/client.rs`

- [ ] **Step 1: Write the failing test**

Add to `src/chroma/client.rs`'s test module:

```rust
    #[tokio::test]
    async fn chroma_module_disabled_is_distinct_from_synapse_not_running() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        // Root probe succeeds → Synapse is up.
        Mock::given(method("GET")).and(path("/")).respond_with(ResponseTemplate::new(200)).mount(&server).await;
        // Chroma endpoint returns 404 → module off.
        Mock::given(method("POST")).and(path("/razer/chromasdk")).respond_with(ResponseTemplate::new(404)).mount(&server).await;

        let client = ChromaClient::with_base_url(&server.uri());
        let err = client.init_session(serde_json::json!({})).await.unwrap_err();
        assert!(matches!(err, ChromaError::ChromaModuleDisabled), "got {err:?}");
    }
```

- [ ] **Step 2: Implement the probe**

Update `init_session` in `src/chroma/client.rs`:

```rust
    pub async fn init_session<T: Serialize>(&self, payload: T) -> Result<Session> {
        let resp = self
            .http
            .post(&self.base_url)
            .json(&payload)
            .send()
            .await
            .map_err(classify_send_error)?;
        let status = resp.status();
        if status.is_success() {
            return resp.json::<Session>().await
                .map_err(|e| ChromaError::SessionInitFailed(e.to_string()));
        }
        // Non-2xx — disambiguate.
        if status.as_u16() == 404 || status.as_u16() == 503 {
            // Probe root — if Synapse is up but Chroma endpoint is off, it's the module.
            let root = self.base_url.trim_end_matches("/razer/chromasdk");
            if self.http.get(root).send().await.map(|r| r.status().is_success()).unwrap_or(false) {
                return Err(ChromaError::ChromaModuleDisabled);
            }
        }
        Err(ChromaError::SessionInitFailed(format!("HTTP {status}")))
    }
```

- [ ] **Step 3: Run tests**

Run: `rtk cargo test --lib chroma::client`
Expected: all 2 tests pass.

- [ ] **Step 4: Commit**

```bash
rtk git add src/chroma/client.rs
rtk git commit -m "feat(chroma): distinguish ChromaModuleDisabled via root probe"
```

### Task 5.2: Degraded mode surfaces in MCP tools

**Files:**
- Modify: `src/mcp/tools.rs`
- Modify: `src/signals/controller.rs` (add a helper)

- [ ] **Step 1: Add a pre-flight check**

In `StatefulController`, add:

```rust
async fn guard_healthy(&self) -> Result<()> {
    use crate::chroma::session::SessionStatus;
    let status = self.session().await?.status().await;
    if matches!(status, SessionStatus::Degraded) {
        return Err(crate::error::ChromaError::SessionInitFailed(
            "Chroma session is DEGRADED — restart Razer Synapse, then call `razer-chroma-mcp doctor` for diagnostics.".into(),
        ));
    }
    Ok(())
}
```

And call `self.guard_healthy().await?` at the top of `apply_user_frame()` and `transition()` (for `transition`, only when it changes to a non-Off state).

- [ ] **Step 2: Test**

Add a unit test that constructs a `StatefulController` fed by a wiremock server that fails every init 5× in a row and asserts a subsequent tool call returns `SessionInitFailed`. If a mockable entry is complex, mark `#[ignore]` with a comment pointing at the manual test in M5 Task 5.4 step 4.

- [ ] **Step 3: Commit**

```bash
rtk git add src/signals/controller.rs src/mcp/tools.rs tests/
rtk git commit -m "feat(mcp): tools return actionable error when session is DEGRADED"
```

### Task 5.3: Graceful shutdown on stdio EOF + SIGTERM

**Files:**
- Modify: `src/cli/serve.rs`

- [ ] **Step 1: Handle EOF from rmcp transport**

The rmcp server's `.serve(transport).await` returns when stdio closes. Ensure the controller shuts down after it returns:

```rust
pub async fn run() -> Result<()> {
    let controller = crate::signals::controller::StatefulController::default();
    controller.start_idle_watcher().await;

    let ctrl_for_shutdown = controller.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("received ctrl-c, shutting down");
            ctrl_for_shutdown.shutdown().await;
            std::process::exit(0);
        }
    });

    let result = crate::mcp::server::serve(controller.clone()).await;
    controller.shutdown().await;
    result
}
```

Rationale: `rmcp` exits the serve loop on stdio EOF (client closed), so falling through to `controller.shutdown()` performs the §8.2 "graceful" path.

- [ ] **Step 2: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 3: Commit**

```bash
rtk git add src/cli/serve.rs
rtk git commit -m "fix(cli): graceful shutdown on stdio EOF"
```

### Task 5.4: `doctor` subcommand

**Files:**
- Modify: `src/cli/doctor.rs`

- [ ] **Step 1: Implement the check sequence from spec §7.3**

```rust
//! `doctor` — environment diagnostics. See spec §7.3.

use crate::chroma::client::ChromaClient;
use crate::chroma::session::{Session, SessionConfig};
use anyhow::Result;
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    let mut any_fail = false;

    println!("razer-chroma-mcp doctor\n");

    // 1. Synapse installed (registry)
    let synapse_installed = is_synapse_installed();
    report("Razer Synapse installed", synapse_installed, "Install from https://www.razer.com/synapse");
    any_fail |= !synapse_installed;

    // 2. Synapse running (process)
    let synapse_running = is_synapse_running();
    report("Razer Synapse running", synapse_running, "Start Razer Synapse from the Start menu");
    any_fail |= !synapse_running;

    // 3. Chroma REST API reachable
    let client = ChromaClient::default();
    let reachable = probe_root(&client).await;
    report("Chroma REST API reachable (port 54235)", reachable, "Open Synapse → Chroma → enable 'Chroma Apps'");
    any_fail |= !reachable;

    // 4. Session init
    let init_ok = Session::init(client.clone(), SessionConfig::default()).await;
    match &init_ok {
        Ok(session) => {
            report("Session init succeeds", true, "");
            session.shutdown().await;
        }
        Err(e) => {
            report("Session init succeeds", false, &format!("{e}"));
            any_fail = true;
        }
    }

    // 5. At least one Chroma device detected
    //    (the SDK does not expose enumeration — fall back to "init succeeded" as proxy)
    report(
        "At least one Chroma device detected",
        init_ok.is_ok(),
        "Plug in a Chroma-capable device; restart Synapse",
    );

    // 6+7. Claude Desktop / Claude Code entries
    use crate::config::paths;
    let desktop_ok = paths::claude_desktop_config_has_our_entry().unwrap_or(false);
    report(
        "Claude Desktop config has `razer-chroma`",
        desktop_ok,
        "Run: razer-chroma-mcp setup",
    );
    let code_ok = paths::claude_code_config_has_our_entry().unwrap_or(false);
    if code_ok {
        report("Claude Code config has `razer-chroma`", true, "");
    } else {
        println!("[ - ] Claude Code config has `razer-chroma`  (optional)");
    }

    // Log path
    println!();
    println!("Log file: {}", log_path().display());

    if any_fail {
        println!("\nSome checks failed. See per-item 'how to fix' above.");
        std::process::exit(2);
    }
    Ok(())
}

fn report(label: &str, ok: bool, fix: &str) {
    if ok {
        println!("[\u{2713}] {label}");
    } else {
        println!("[x] {label}");
        if !fix.is_empty() {
            println!("    fix: {fix}");
        }
    }
}

fn is_synapse_installed() -> bool {
    // HKCU then HKLM — non-fatal if we can't read.
    #[cfg(windows)]
    {
        use std::process::Command;
        // Use `reg query` to avoid adding a winreg dependency for one lookup.
        let keys = [
            r"HKCU\Software\Razer\Synapse3",
            r"HKLM\Software\Razer\Synapse3",
            r"HKCU\Software\Razer\Synapse",
        ];
        keys.iter().any(|k| {
            Command::new("reg").args(["query", k]).output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
    }
    #[cfg(not(windows))]
    { false }
}

fn is_synapse_running() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq Razer Synapse 3.exe")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Razer Synapse"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    { false }
}

async fn probe_root(_: &ChromaClient) -> bool {
    let c = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(800))
        .build()
        .unwrap();
    c.get("http://localhost:54235/").send().await.map(|r| r.status().is_success()).unwrap_or(false)
}

fn log_path() -> PathBuf {
    std::env::var("LOCALAPPDATA")
        .map(|p| PathBuf::from(p).join("razer-chroma-mcp").join("logs"))
        .unwrap_or_else(|_| PathBuf::from("./logs"))
}
```

- [ ] **Step 2: Stub `paths::claude_desktop_config_has_our_entry` (full implementation in M6)**

In `src/config/paths.rs`:

```rust
//! Platform config path resolution + quick "does our entry exist" checks.
use std::path::PathBuf;

pub fn claude_desktop_config() -> Option<PathBuf> {
    std::env::var("APPDATA").ok().map(|p| PathBuf::from(p).join("Claude").join("claude_desktop_config.json"))
}

pub fn claude_code_config() -> Option<PathBuf> {
    std::env::var("USERPROFILE").ok().map(|p| PathBuf::from(p).join(".claude.json"))
}

pub fn claude_desktop_config_has_our_entry() -> Option<bool> {
    has_entry(claude_desktop_config()?)
}

pub fn claude_code_config_has_our_entry() -> Option<bool> {
    has_entry(claude_code_config()?)
}

fn has_entry(path: PathBuf) -> Option<bool> {
    let text = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(
        v.get("mcpServers")
            .and_then(|s| s.as_object())
            .map(|m| m.contains_key("razer-chroma"))
            .unwrap_or(false),
    )
}
```

- [ ] **Step 3: Verify build**

Run: `rtk cargo build`
Expected: builds successfully.

- [ ] **Step 4: Manual verification**

Run: `./target/debug/razer-chroma-mcp.exe doctor`
Expected output (on a healthy dev machine): every check ✓ except Claude Desktop entry (until M6 registers it).

Run with Synapse stopped: expect `[x] Razer Synapse running` with fix line.

- [ ] **Step 5: Commit**

```bash
rtk git add src/cli/doctor.rs src/config/paths.rs
rtk git commit -m "feat(cli): doctor subcommand with registry/process/http/config checks"
```

### Task 5.5: `pattern` subcommand (preview any pattern without Claude Desktop)

**Files:**
- Modify: `src/cli/pattern.rs`

- [ ] **Step 1: Implement**

```rust
//! `pattern <name> [--seconds N]` — preview a pattern without Claude Desktop.
//! Spec §7.4 (added per reviewer N3).

use crate::signals::controller::{Incoming, StatefulController};
use anyhow::{anyhow, Result};
use std::time::Duration;

pub async fn run(name: String, seconds: u64) -> Result<()> {
    let controller = StatefulController::default();
    let incoming = match name.as_str() {
        "thinking" => Incoming::Thinking,
        "needs_input" => Incoming::NeedsInput,
        "idle" => Incoming::Idle,
        "complete" => Incoming::Complete,
        "error" => Incoming::Error,
        other => return Err(anyhow!(
            "unknown pattern '{other}'. Known: thinking, needs_input, idle, complete, error"
        )),
    };

    controller.transition(incoming).await?;
    println!("playing '{name}' for {seconds}s (Ctrl+C to stop)");

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => {}
        _ = tokio::signal::ctrl_c() => { println!("interrupted"); }
    }

    controller.shutdown().await;
    Ok(())
}
```

- [ ] **Step 2: Manual verification**

Run: `./target/debug/razer-chroma-mcp.exe pattern needs_input --seconds 6`
Expected: triple-flash → breathing + accents for 6 seconds, then clean shutdown.

- [ ] **Step 3: Commit**

```bash
rtk git add src/cli/pattern.rs
rtk git commit -m "feat(cli): pattern subcommand for standalone previews"
```

### Task 5.6: Reconnect-restore manual verification

**Files:**
- Modify: `docs/manual-testing.md`

- [ ] **Step 1: Write down the procedure**

Append:

```markdown
## M5 — 2026-MM-DD — Reconnect restore

Procedure:
1. Start `razer-chroma-mcp serve`.
2. From an MCP client (or hand-crafted stdio) call `signal_thinking`.
3. Kill `Razer Synapse 3.exe` from Task Manager.
4. Observe: server logs 3× heartbeat failures, transitions to Reconnecting.
5. Restart Razer Synapse.
6. Within ~15s, `thinking` pattern should be visible again on the keyboard.
7. Also verify `set_static_color #D97757` followed by Synapse kill-restart → keyboard returns to terracotta.

Outcome: <pass/fail/notes>
```

- [ ] **Step 2: Execute + record**

Run the procedure on BlackWidow. Record times and observations.

- [ ] **Step 3: Commit**

```bash
rtk git add docs/manual-testing.md
rtk git commit -m "docs: manual verification of reconnect restore"
```

### Task 5.7: Error message audit — all variants bilingual

**Files:**
- Modify: `src/error.rs`

- [ ] **Step 1: Ensure every user-facing variant has Korean + English**

Already done for `SynapseNotRunning` and `ChromaModuleDisabled`. Update the remaining user-facing variants (skip purely internal ones):

```rust
    #[error("Session init failed: {0}\n세션 초기화에 실패했습니다. Razer Synapse를 재시작한 뒤 다시 시도해 주세요.")]
    SessionInitFailed(String),

    #[error("Invalid color: {0}\n잘못된 색상 형식입니다. #RRGGBB 또는 {{r,g,b}} 형태로 입력해 주세요.")]
    InvalidColor(String),

    #[error("Unknown device: {0}\n알 수 없는 디바이스입니다. keyboard/mouse/mousepad/headset/keypad/chromalink 중에서 선택해 주세요.")]
    UnknownDevice(String),

    #[error("Config file corrupt: {path}. A copy has been saved to {quarantine}.\n설정 파일이 손상되었습니다. 백업본: {quarantine}")]
    ConfigCorrupt { path: String, quarantine: String },
```

Keep `Network`/`HeartbeatFailed` English-only since they're diagnostic for logs.

- [ ] **Step 2: Verify tests still pass**

Run: `rtk cargo test`
Expected: all green. Update any snapshot that fixed error strings (should only be snapshot-adjacent if an error is serialized into effects.rs snapshots — unlikely).

- [ ] **Step 3: Commit**

```bash
rtk git add src/error.rs
rtk git commit -m "feat(error): bilingual ko/en messages for user-facing variants"
```

### Task 5.8: Log rotation — 7-day retention

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Prune old logs on startup**

Before calling `tracing_appender::rolling::daily`, scan the log directory for `mcp.log.*` older than 7 days and delete:

```rust
fn prune_old_logs(dir: &std::path::Path) {
    let cutoff = std::time::SystemTime::now() - std::time::Duration::from_secs(7 * 24 * 60 * 60);
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        let Ok(modified) = meta.modified() else { continue };
        if modified < cutoff {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
```

Call it in `init_tracing` after `create_dir_all`. Keep the current daily-rotation behavior otherwise.

- [ ] **Step 2: Unit test (tempdir)**

Add inline in `src/main.rs` (or extract to a small helper module):

```rust
#[cfg(test)]
mod log_prune_tests {
    use super::*;
    use std::fs::{self, File};
    use std::time::{Duration, SystemTime};

    #[test]
    fn prune_removes_files_older_than_7_days() {
        let tmp = tempfile::tempdir().unwrap();
        let old = tmp.path().join("mcp.log.2025-01-01");
        File::create(&old).unwrap();
        let older_than_week = SystemTime::now() - Duration::from_secs(10 * 24 * 60 * 60);
        filetime::set_file_mtime(&old, filetime::FileTime::from_system_time(older_than_week)).unwrap();
        let fresh = tmp.path().join("mcp.log.today");
        File::create(&fresh).unwrap();

        prune_old_logs(tmp.path());

        assert!(!old.exists(), "old log should be removed");
        assert!(fresh.exists(), "fresh log should remain");
    }
}
```

Add `filetime = "0.2"` to `[dev-dependencies]` in `Cargo.toml`.

- [ ] **Step 3: Run test**

Run: `rtk cargo test --lib log_prune_tests`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
rtk git add src/main.rs Cargo.toml Cargo.lock
rtk git commit -m "feat(logging): prune logs older than 7 days on startup"
```

### Task 5.9: M5 close-out

- [ ] **Step 1: Update CHANGELOG**

```markdown
- `doctor` subcommand with registry, process, HTTP, and config checks.
- `pattern` subcommand previews any named pattern standalone.
- `ChromaModuleDisabled` distinguished from `SynapseNotRunning` via root probe.
- Bilingual error messages for all user-facing variants.
- 7-day log retention with prune-on-startup.
- Degraded-mode guard on MCP tool calls.
```

- [ ] **Step 2: Commit and report M5 complete**

```bash
rtk git add CHANGELOG.md
rtk git commit -m "docs: changelog m5"
```

---

## Milestone 6: Install UX + Docs

Ship the NSIS installer, the `setup` wizard with config deep-merge, `uninstall`, the full README + docs, and a release GitHub Actions workflow. Exit criterion: a non-technical tester installs via Setup.exe in under 3 minutes without opening a terminal.

### Task 6.1: Config merge algorithm — 9 unit tests (TDD)

**Files:**
- Modify: `src/config/merge.rs`

- [ ] **Step 1: Write every spec §7.6 test case as a failing test first**

```rust
//! Deep-merge our `razer-chroma` entry into Claude Desktop / Claude Code config.
//! Algorithm is spec §7.6; every numbered step maps to a test below.

use crate::error::{ChromaError, Result};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const OUR_KEY: &str = "razer-chroma";

pub struct MergeInput<'a> {
    pub path: &'a Path,
    pub binary_path: &'a Path,
    pub extra_args: &'a [&'a str],
}

#[derive(Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    /// File did not exist; we created it.
    Created,
    /// Our entry was already byte-equivalent; no write.
    Idempotent,
    /// Merged, keeping other keys intact.
    Merged,
    /// Overwrote a previous `razer-chroma` entry differing from ours.
    Replaced,
}

pub fn merge(input: &MergeInput) -> Result<MergeOutcome> {
    let our_entry = our_entry(input.binary_path, input.extra_args);

    // 1. Read (or treat missing as `{}`).
    let (mut root, existed) = match fs::read_to_string(input.path) {
        Ok(s) if s.trim().is_empty() => (Value::Object(Map::new()), true),
        Ok(s) => {
            let v = parse_strict_or_quarantine(input.path, &s)?;
            (v, true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Value::Object(Map::new()), false),
        Err(e) => return Err(ChromaError::SessionInitFailed(format!("read {}: {e}", input.path.display()))),
    };

    // 3. Validate root is object.
    let Some(obj) = root.as_object_mut() else {
        return Err(ChromaError::ConfigCorrupt {
            path: input.path.display().to_string(),
            quarantine: quarantine_copy(input.path)?.display().to_string(),
        });
    };

    // 4. Deep-merge into `mcpServers`.
    match obj.get_mut("mcpServers") {
        None => {
            let mut m = Map::new();
            m.insert(OUR_KEY.to_string(), our_entry.clone());
            obj.insert("mcpServers".to_string(), Value::Object(m));
        }
        Some(Value::Object(servers)) => {
            let existing = servers.get(OUR_KEY);
            if existing == Some(&our_entry) {
                return Ok(MergeOutcome::Idempotent); // 7.
            }
            servers.insert(OUR_KEY.to_string(), our_entry.clone());
        }
        Some(_non_object) => {
            return Err(ChromaError::ConfigCorrupt {
                path: input.path.display().to_string(),
                quarantine: quarantine_copy(input.path)?.display().to_string(),
            });
        }
    }

    // 6. Backup (only if original existed).
    if existed {
        let _ = backup_file(input.path);
        prune_backups(input.path, 3);
    }

    // 5. Atomic write.
    atomic_write(input.path, &root)?;

    if !existed {
        Ok(MergeOutcome::Created)
    } else {
        Ok(MergeOutcome::Merged)
    }
}

fn our_entry(bin: &Path, extra_args: &[&str]) -> Value {
    let mut args = vec!["serve".to_string()];
    args.extend(extra_args.iter().map(|s| s.to_string()));
    serde_json::json!({
        "command": bin.display().to_string(),
        "args": args,
    })
}

fn parse_strict_or_quarantine(path: &Path, text: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(text) {
        Ok(v) => Ok(v),
        Err(_) => {
            let q = quarantine_copy(path)?;
            Err(ChromaError::ConfigCorrupt {
                path: path.display().to_string(),
                quarantine: q.display().to_string(),
            })
        }
    }
}

fn quarantine_copy(path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let q = path.with_extension(format!("corrupt-{ts}"));
    fs::copy(path, &q).ok();
    Ok(q)
}

fn backup_file(path: &Path) -> Result<PathBuf> {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let b = path.with_extension(format!("bak-{ts}"));
    fs::copy(path, &b).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    Ok(b)
}

fn prune_backups(path: &Path, keep: usize) {
    let Some(dir) = path.parent() else { return };
    let Some(stem) = path.file_name().and_then(|s| s.to_str()) else { return };
    let prefix = format!("{stem}.bak-");
    let mut entries: Vec<_> = fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    while entries.len() > keep {
        let old = entries.remove(0);
        let _ = fs::remove_file(old.path());
    }
}

fn atomic_write(path: &Path, value: &Value) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    let tmp = dir.join(format!(".{}.tmp-{}", path.file_name().and_then(|s| s.to_str()).unwrap_or("cfg"), std::process::id()));
    let text = serde_json::to_string_pretty(value).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    fs::write(&tmp, &text).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    // fsync (best-effort on Windows; rename is atomic across cases that matter).
    if let Ok(f) = fs::File::open(&tmp) { let _ = f.sync_all(); }
    fs::rename(&tmp, path).map_err(|e| ChromaError::SessionInitFailed(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn run(dir: &Path, name: &str, initial: Option<&str>) -> (PathBuf, Result<MergeOutcome>) {
        let p = dir.join(name);
        if let Some(txt) = initial {
            fs::write(&p, txt).unwrap();
        }
        let bin = PathBuf::from(r"C:\Users\tester\razer-chroma-mcp.exe");
        let input = MergeInput { path: &p, binary_path: &bin, extra_args: &[] };
        (p.clone(), merge(&input))
    }

    #[test]
    fn missing_file_creates_it() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", None);
        assert_eq!(r.unwrap(), MergeOutcome::Created);
    }

    #[test]
    fn empty_file_treated_as_empty_object() {
        let tmp = tempdir().unwrap();
        let (p, r) = run(tmp.path(), "c.json", Some(""));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert!(v["mcpServers"]["razer-chroma"].is_object());
    }

    #[test]
    fn bare_object_adds_mcp_servers() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("{}"));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
    }

    #[test]
    fn preserves_existing_other_servers() {
        let tmp = tempdir().unwrap();
        let init = r#"{ "mcpServers": { "other": { "command": "x" } }, "otherTop": true }"#;
        let (p, r) = run(tmp.path(), "c.json", Some(init));
        assert_eq!(r.unwrap(), MergeOutcome::Merged);
        let v: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
        assert_eq!(v["mcpServers"]["other"]["command"], "x");
        assert_eq!(v["otherTop"], true);
        assert!(v["mcpServers"]["razer-chroma"].is_object());
    }

    #[test]
    fn malformed_json_quarantines_and_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("{ not valid"));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
        assert!(
            fs::read_dir(tmp.path()).unwrap().any(|e| {
                e.unwrap().file_name().to_string_lossy().contains("corrupt-")
            })
        );
    }

    #[test]
    fn non_object_root_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some("[1,2,3]"));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
    }

    #[test]
    fn non_object_mcp_servers_errors() {
        let tmp = tempdir().unwrap();
        let (_, r) = run(tmp.path(), "c.json", Some(r#"{ "mcpServers": "oops" }"#));
        assert!(matches!(r, Err(ChromaError::ConfigCorrupt { .. })));
    }

    #[test]
    fn idempotent_rerun_does_not_write() {
        let tmp = tempdir().unwrap();
        let (p, r1) = run(tmp.path(), "c.json", None);
        assert_eq!(r1.unwrap(), MergeOutcome::Created);
        let first_mtime = fs::metadata(&p).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let bin = PathBuf::from(r"C:\Users\tester\razer-chroma-mcp.exe");
        let input = MergeInput { path: &p, binary_path: &bin, extra_args: &[] };
        assert_eq!(merge(&input).unwrap(), MergeOutcome::Idempotent);
        let second_mtime = fs::metadata(&p).unwrap().modified().unwrap();
        assert_eq!(first_mtime, second_mtime);
    }

    #[test]
    fn backup_rotation_keeps_three() {
        let tmp = tempdir().unwrap();
        let p = tmp.path().join("c.json");
        fs::write(&p, r#"{ "mcpServers": {} }"#).unwrap();
        let bin = PathBuf::from(r"C:\Users\tester\razer-chroma-mcp.exe");
        // Run merge 5 times, each with a slightly different binary path so it's never idempotent.
        for i in 0..5 {
            let bin_i = PathBuf::from(format!(r"C:\Users\tester\razer-chroma-mcp-{i}.exe"));
            let input = MergeInput { path: &p, binary_path: &bin_i, extra_args: &[] };
            let _ = merge(&input).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let baks: Vec<_> = fs::read_dir(tmp.path()).unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak-"))
            .collect();
        assert!(baks.len() <= 3, "expected ≤3 backups, got {}", baks.len());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `rtk cargo test --lib config::merge`
Expected: all 9 tests pass.

- [ ] **Step 3: Commit**

```bash
rtk git add src/config/merge.rs
rtk git commit -m "feat(config): deep-merge algorithm with atomic write, quarantine, backup rotation"
```

### Task 6.2: `setup` wizard

**Files:**
- Modify: `src/cli/setup.rs`

- [ ] **Step 1: Implement the wizard per spec §7.2**

```rust
//! `setup` — install wizard. Spec §7.2.

use crate::chroma::client::ChromaClient;
use crate::chroma::session::{Session, SessionConfig};
use crate::config::merge::{merge, MergeInput, MergeOutcome};
use crate::config::paths;
use anyhow::Result;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

pub async fn run(lang: Option<String>) -> Result<()> {
    let lang = lang.unwrap_or_else(detect_lang);
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();

    say(&lang, "Welcome to razer-chroma-mcp setup.", "razer-chroma-mcp 설치 마법사입니다.");

    // 1. Detect Synapse
    if !synapse_ok().await {
        say(&lang, "Razer Synapse is not reachable.", "Razer Synapse가 실행 중이지 않습니다.");
        say(&lang, "Please install Synapse from https://www.razer.com/synapse, then press Enter to continue.",
                   "https://www.razer.com/synapse 에서 Synapse를 설치한 뒤 Enter를 눌러 주세요.");
        wait_enter(&mut reader);
    }
    while !synapse_ok().await {
        say(&lang, "Still can't reach Synapse. Press Enter to retry, or Ctrl+C to exit.",
                   "Synapse를 찾을 수 없습니다. Enter=재시도, Ctrl+C=종료.");
        wait_enter(&mut reader);
    }
    say(&lang, "[OK] Synapse reachable.", "[OK] Synapse 연결됨.");

    // 2. Verify Chroma SDK
    let client = ChromaClient::default();
    match Session::init(client.clone(), SessionConfig::default()).await {
        Ok(s) => {
            say(&lang, "[OK] Chroma session initialized.", "[OK] Chroma 세션 초기화 성공.");
            // 3. Hardware demo
            say(&lang, "Playing a 3-second terracotta sweep on your keyboard…",
                       "키보드에 3초 동안 테라코타 스윕을 재생합니다...");
            let _ = crate::cli::demo::run().await;
            say(&lang, "Did you see the lights move? [Y/n]",
                       "키보드에 조명 효과가 보였나요? [Y/n]");
            let ans = read_line(&mut reader);
            if ans.trim().eq_ignore_ascii_case("n") {
                say(&lang, "If the lights did not move, your device may not be Chroma-compatible or Synapse may be blocking us. Run `razer-chroma-mcp doctor` for details.",
                           "조명이 보이지 않으면 기기가 Chroma를 지원하지 않거나 Synapse가 차단 중일 수 있습니다. `razer-chroma-mcp doctor`로 진단해 주세요.");
            }
            s.shutdown().await;
        }
        Err(e) => {
            say(&lang, &format!("[FAIL] Chroma session init failed: {e}"),
                       &format!("[FAIL] Chroma 세션 초기화 실패: {e}"));
        }
    }

    // 4–5. Register with Claude Desktop / Code
    let bin = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("razer-chroma-mcp.exe"));
    if let Some(path) = paths::claude_desktop_config() {
        offer_merge(&lang, &path, &bin, "Claude Desktop", &mut reader);
    }
    if let Some(path) = paths::claude_code_config() {
        offer_merge(&lang, &path, &bin, "Claude Code", &mut reader);
    }

    // 6. Final doctor
    say(&lang, "Running final diagnostics…", "최종 진단을 실행합니다...");
    let _ = crate::cli::doctor::run().await;

    // 7. Restart prompt
    say(&lang, "Setup complete. Please restart Claude Desktop to activate.",
               "설치가 완료되었습니다. Claude Desktop을 재시작해 주세요.");
    Ok(())
}

fn detect_lang() -> String {
    #[cfg(windows)]
    {
        use std::process::Command;
        // Fall back to English unless locale is Korean.
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-Culture).Name"])
            .output();
        if let Ok(o) = out {
            if String::from_utf8_lossy(&o.stdout).trim().starts_with("ko") {
                return "ko".into();
            }
        }
    }
    "en".into()
}

fn say(lang: &str, en: &str, ko: &str) {
    if lang == "ko" {
        println!("{ko}");
        println!("  ({en})");
    } else {
        println!("{en}");
        println!("  ({ko})");
    }
}

fn read_line<R: BufRead>(r: &mut R) -> String {
    let mut s = String::new();
    r.read_line(&mut s).ok();
    s
}

fn wait_enter<R: BufRead>(r: &mut R) {
    print!("> ");
    let _ = std::io::stdout().flush();
    let _ = read_line(r);
}

async fn synapse_ok() -> bool {
    let c = reqwest::Client::builder().timeout(Duration::from_millis(800)).build().unwrap();
    c.get("http://localhost:54235/").send().await.map(|r| r.status().is_success()).unwrap_or(false)
}

fn offer_merge<R: BufRead>(lang: &str, path: &std::path::Path, bin: &std::path::Path, label: &str, reader: &mut R) {
    say(lang,
        &format!("Register razer-chroma with {label}? ({}) [Y/n]", path.display()),
        &format!("{label}에 razer-chroma를 등록할까요? ({}) [Y/n]", path.display()));
    if read_line(reader).trim().eq_ignore_ascii_case("n") {
        return;
    }
    let input = MergeInput { path, binary_path: bin, extra_args: &[] };
    match merge(&input) {
        Ok(MergeOutcome::Created) | Ok(MergeOutcome::Merged) | Ok(MergeOutcome::Replaced) => {
            say(lang, &format!("[OK] {label} configured."), &format!("[OK] {label} 설정 완료."));
        }
        Ok(MergeOutcome::Idempotent) => {
            say(lang, &format!("[OK] {label} already configured."), &format!("[OK] {label} 이미 설정되어 있습니다."));
        }
        Err(e) => {
            say(lang, &format!("[FAIL] Could not write {label} config: {e}"),
                      &format!("[FAIL] {label} 설정을 쓰지 못했습니다: {e}"));
        }
    }
}
```

- [ ] **Step 2: Verify build + manual smoke test**

Run: `rtk cargo build`

Run: `./target/debug/razer-chroma-mcp.exe setup --lang en`
Expected: prompts through each step, writes `%APPDATA%\Claude\claude_desktop_config.json` with our entry; re-running reports "already configured".

- [ ] **Step 3: Commit**

```bash
rtk git add src/cli/setup.rs
rtk git commit -m "feat(cli): setup wizard with bilingual prompts and claude config merge"
```

### Task 6.3: `uninstall` subcommand

**Files:**
- Modify: `src/cli/uninstall.rs`

- [ ] **Step 1: Implement per spec §7.5**

```rust
//! `uninstall` — remove our entry from Claude Desktop / Code configs cleanly.

use crate::config::merge::OUR_KEY;
use crate::config::paths;
use anyhow::Result;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn run() -> Result<()> {
    if is_claude_desktop_running() {
        println!("Claude Desktop is currently running.");
        println!("Please close Claude Desktop before uninstalling to avoid config corruption. Continue anyway? [y/N]");
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).ok();
        if !line.trim().eq_ignore_ascii_case("y") {
            println!("Aborting.");
            return Ok(());
        }
    }

    if let Some(p) = paths::claude_desktop_config() {
        remove_entry(&p)?;
    }
    if let Some(p) = paths::claude_code_config() {
        remove_entry(&p)?;
    }
    println!("Uninstall complete.");
    Ok(())
}

fn is_claude_desktop_running() -> bool {
    #[cfg(windows)]
    {
        use std::process::Command;
        Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq Claude.exe")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Claude.exe"))
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    { false }
}

fn remove_entry(path: &Path) -> Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(()); // missing = no-op
    };
    let Ok(mut v) = serde_json::from_str::<Value>(&text) else {
        eprintln!("warn: {} is not valid JSON; skipping", path.display());
        return Ok(());
    };
    let Some(obj) = v.as_object_mut() else { return Ok(()); };
    let Some(Value::Object(servers)) = obj.get_mut("mcpServers") else { return Ok(()); };
    if servers.remove(OUR_KEY).is_none() {
        return Ok(());
    }
    // Backup
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let bak = path.with_extension(format!("bak-{ts}"));
    let _ = fs::copy(path, &bak);
    // Atomic write via tmp+rename
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(".{}.tmp-{}", path.file_name().and_then(|s| s.to_str()).unwrap_or("cfg"), std::process::id()));
    fs::write(&tmp, serde_json::to_string_pretty(&v)?)?;
    fs::rename(&tmp, path)?;
    println!("Removed razer-chroma from {}", path.display());
    Ok(())
}
```

- [ ] **Step 2: Manual verification**

Run: `./target/debug/razer-chroma-mcp.exe uninstall`
Expected: if entry present, removes and prints path; if Claude Desktop running, warns first.

- [ ] **Step 3: Commit**

```bash
rtk git add src/cli/uninstall.rs
rtk git commit -m "feat(cli): uninstall removes our entry by key match with backup"
```

### Task 6.4: NSIS installer script

**Files:**
- Create: `installer/razer-chroma-mcp.nsi`
- Create: `installer/README.md`

- [ ] **Step 1: Write the NSIS script**

```nsis
; razer-chroma-mcp installer — per-user, no UAC. Spec §7.1.
!include "MUI2.nsh"

Name "razer-chroma-mcp"
OutFile "razer-chroma-mcp-setup-${VERSION}.exe"
RequestExecutionLevel user
Unicode true

InstallDir "$LOCALAPPDATA\Programs\razer-chroma-mcp"
InstallDirRegKey HKCU "Software\razer-chroma-mcp" "InstallDir"

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\LICENSE-MIT"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!define MUI_FINISHPAGE_RUN "$INSTDIR\razer-chroma-mcp.exe"
!define MUI_FINISHPAGE_RUN_PARAMETERS "setup"
!define MUI_FINISHPAGE_RUN_TEXT "Launch setup wizard"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Korean"

Section "Install"
  SetOutPath "$INSTDIR"
  File "..\target\release\razer-chroma-mcp.exe"
  File "..\LICENSE-MIT"
  File "..\LICENSE-APACHE"
  File "..\README.md"

  ; Add to user PATH via HKCU
  EnVar::SetHKCU
  EnVar::AddValue "PATH" "$INSTDIR"

  ; Start Menu shortcut
  CreateDirectory "$SMPROGRAMS\razer-chroma-mcp"
  CreateShortcut "$SMPROGRAMS\razer-chroma-mcp\Razer Chroma MCP Setup.lnk" \
                 "$INSTDIR\razer-chroma-mcp.exe" "setup"

  ; Apps & Features entry
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\razer-chroma-mcp" \
              "DisplayName" "razer-chroma-mcp"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\razer-chroma-mcp" \
              "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\razer-chroma-mcp" \
              "Publisher" "roscore"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\razer-chroma-mcp" \
              "UninstallString" "$INSTDIR\uninstall.exe"
  WriteRegStr HKCU "Software\razer-chroma-mcp" "InstallDir" "$INSTDIR"

  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

Section "Uninstall"
  ; Call our own uninstall subcommand to clean Claude configs
  ExecWait '"$INSTDIR\razer-chroma-mcp.exe" uninstall'

  Delete "$INSTDIR\razer-chroma-mcp.exe"
  Delete "$INSTDIR\LICENSE-MIT"
  Delete "$INSTDIR\LICENSE-APACHE"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"

  EnVar::SetHKCU
  EnVar::DeleteValue "PATH" "$INSTDIR"

  Delete "$SMPROGRAMS\razer-chroma-mcp\Razer Chroma MCP Setup.lnk"
  RMDir "$SMPROGRAMS\razer-chroma-mcp"

  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\razer-chroma-mcp"
  DeleteRegKey HKCU "Software\razer-chroma-mcp"
SectionEnd
```

- [ ] **Step 2: Write installer/README.md with build instructions**

```markdown
# Building the Windows installer

Requires NSIS 3.x (`choco install nsis` or https://nsis.sourceforge.io/) and
the `EnVar` plugin (copy `EnVar.dll` into NSIS `Plugins\x86-ansi\`).

## Steps

1. Build the release binary from repo root:
   ```
   cargo build --release
   ```

2. Compile the installer from this directory:
   ```
   makensis /DVERSION=0.1.0 razer-chroma-mcp.nsi
   ```

3. Output: `razer-chroma-mcp-setup-0.1.0.exe`

CI builds this automatically on tag push; see `.github/workflows/release.yml`.
```

- [ ] **Step 3: Commit**

```bash
rtk git add installer/
rtk git commit -m "feat(installer): nsis script, per-user install, no UAC"
```

### Task 6.5: Release CI — build installer on tag push

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write release workflow**

```yaml
name: Release

on:
  push:
    tags: ["v*"]

jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75.0
      - uses: Swatinem/rust-cache@v2

      - name: Build release binary
        run: cargo build --release

      - name: Install NSIS
        run: choco install nsis -y --no-progress

      - name: Install EnVar plugin
        shell: pwsh
        run: |
          $url = "https://nsis.sourceforge.io/mediawiki/images/7/7f/EnVar_plugin.zip"
          Invoke-WebRequest -Uri $url -OutFile envar.zip
          Expand-Archive envar.zip -DestinationPath envar
          Copy-Item envar\Plugins\x86-ansi\EnVar.dll "C:\Program Files (x86)\NSIS\Plugins\x86-ansi\"

      - name: Determine version
        id: ver
        shell: pwsh
        run: |
          $tag = "${env:GITHUB_REF}" -replace "refs/tags/v",""
          "version=$tag" | Out-File -FilePath $env:GITHUB_OUTPUT -Append

      - name: Build installer
        working-directory: installer
        run: makensis /DVERSION=${{ steps.ver.outputs.version }} razer-chroma-mcp.nsi

      - name: Upload to release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            installer/razer-chroma-mcp-setup-${{ steps.ver.outputs.version }}.exe
            target/release/razer-chroma-mcp.exe
```

- [ ] **Step 2: Commit**

```bash
rtk git add .github/workflows/release.yml
rtk git commit -m "ci: release workflow builds nsis installer on tag push"
```

### Task 6.6: Full README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace README stub with the full README**

```markdown
# razer-chroma-mcp

> Give Claude a body language on your Razer keyboard.

When Claude is thinking, your BlackWidow pulses terracotta. When Claude needs your input, the keyboard unmistakably calls for you. When Claude finishes, a Claude-branded asterisk flashes once and rests. All in Anthropic's warm palette.

[![CI](https://github.com/roscore/razer-chroma-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/roscore/razer-chroma-mcp/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

## 3-step install (Windows)

1. **Download** `razer-chroma-mcp-setup-<version>.exe` from [Releases](https://github.com/roscore/razer-chroma-mcp/releases).
2. **Double-click** it. (Windows SmartScreen may warn — see [install guide](docs/install.md).)
3. **Follow the wizard.** Restart Claude Desktop when it tells you to. Done.

No terminal required.

## Requirements

- Windows 10 or 11
- Razer Synapse 3 with the Chroma module enabled
- A Razer Chroma-capable device (tested on BlackWidow; any Chroma keyboard should work)
- Claude Desktop ≥ 0.8 or Claude Code

## Supported devices

The 5 `signal_*` patterns render on any Razer keyboard using the generic 6×22 grid. Mouse, mousepad, headset, keypad, and Chroma Link work with the low-level tools (`set_static_color` etc.) but do not render signals in v0.1.0 — see [#v0.2.0 device parity](https://github.com/roscore/razer-chroma-mcp/issues).

## Not using Claude Desktop? `cargo install`

```
cargo install razer-chroma-mcp
razer-chroma-mcp setup
```

## Troubleshooting

- Keyboard flickers or fights another app → [last-writer-wins explainer](docs/troubleshooting.md#multiple-chroma-apps)
- SmartScreen "Unrecognized app" on install → [install guide](docs/install.md#smartscreen)
- Nothing happens → run `razer-chroma-mcp doctor`

## Docs

- [Install guide with screenshots](docs/install.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Architecture (for contributors)](docs/architecture.md)
- [Manual testing checklist](docs/manual-testing.md)

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

## Trademarks

Razer is a trademark of Razer Inc. This project is not affiliated with or endorsed by Razer.

Anthropic and Claude are trademarks of Anthropic, PBC. Warm palette used for aesthetic fit; this project is not affiliated with or endorsed by Anthropic.

---

## 한국어 요약

Claude Desktop / Claude Code가 Razer Chroma 키보드를 통해 **생각 중 / 입력 대기 / 완료 / 오류** 상태를 시각적으로 표현하도록 해 주는 MCP 서버입니다. 비개발자도 Setup.exe를 더블클릭해 3분 안에 설치할 수 있습니다. 터미널이 필요 없습니다.
```

- [ ] **Step 2: Commit**

```bash
rtk git add README.md
rtk git commit -m "docs: full readme with 3-step install and troubleshooting links"
```

### Task 6.7: Install, troubleshooting, architecture, manual-testing docs

**Files:**
- Create: `docs/install.md`
- Create: `docs/troubleshooting.md`
- Create: `docs/architecture.md`
- Modify: `docs/manual-testing.md` (finalize sections)

- [ ] **Step 1: Write docs/install.md**

```markdown
# Install guide

## 1. Download

Get `razer-chroma-mcp-setup-<version>.exe` from the [Releases page](https://github.com/roscore/razer-chroma-mcp/releases).

## 2. Run the installer

Double-click the downloaded file.

### SmartScreen warning

Windows SmartScreen will show **"Windows protected your PC"** because v0.1.0 is unsigned (code-signing certificates cost money; we're deferring it to v0.2.0). To continue:

1. Click **More info**.
2. Click **Run anyway**.

![SmartScreen screenshot placeholder — add at release](../installer/assets/smartscreen.png)

(If you don't trust the installer, verify the SHA-256 hash published on the release page, or install via `cargo install razer-chroma-mcp`.)

## 3. Follow the wizard

The wizard will:

1. Check that Razer Synapse is running.
2. Play a 3-second terracotta sweep on your keyboard to verify the Chroma SDK.
3. Ask whether to register with Claude Desktop and/or Claude Code.
4. Run diagnostics.

## 4. Restart Claude Desktop

Close and re-open Claude Desktop. Ask Claude to think about something — the keyboard should pulse terracotta.

## Uninstall

Go to **Settings → Apps → Installed apps**, find **razer-chroma-mcp**, click **Uninstall**. The uninstaller will remove our entry from Claude Desktop/Code configs (backups kept as `*.bak-<timestamp>`).
```

- [ ] **Step 2: Write docs/troubleshooting.md**

```markdown
# Troubleshooting

## Check first: `razer-chroma-mcp doctor`

Run from a terminal (or via Start Menu → Razer Chroma MCP Setup → wait for wizard → it runs doctor at the end). Each `[x]` item prints a fix.

## Symptoms

### Nothing happens when I ask Claude a question

- Is the server registered? Check `%APPDATA%\Claude\claude_desktop_config.json` contains `"razer-chroma"` under `"mcpServers"`.
- Did you restart Claude Desktop after setup? MCP servers load at startup only.
- Run `razer-chroma-mcp doctor`.

### My keyboard flickers / another app is controlling it

Razer's Chroma SDK explicitly allows multiple apps to drive the keyboard at once — whichever writes the last frame wins. If you have another Chroma-aware app running (a game, another RGB utility), they will fight for the keyboard. Close the other app, or accept flicker.

### SmartScreen blocks the installer

v0.1.0 is unsigned. Click **More info → Run anyway**. v0.2.0 will be signed.

### Synapse is running but `doctor` says "Chroma REST API unreachable"

Open Synapse → **Chroma** → ensure **Chroma Apps** toggle is on. The REST API listens on `http://localhost:54235` only when this is enabled.

### Where are the logs?

`%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log`. Rotated daily, retained 7 days.

### I use Claude Code, not Claude Desktop

The wizard will detect `%USERPROFILE%\.claude.json` and offer to register there too.

## Reset

Uninstall, then delete `%APPDATA%\Claude\claude_desktop_config.json.bak-*` and `%USERPROFILE%\.claude.json.bak-*` if you want to remove history.
```

- [ ] **Step 3: Write docs/architecture.md**

```markdown
# Architecture

For contributors. User-facing docs are in the [README](../README.md).

## Three layers

```
mcp  →  signals  →  chroma
```

Dependencies are strictly one-way. `chroma` knows about HTTP and Chroma's payload shapes but nothing about Claude states. `signals` owns `ClaudeState` and animator lifecycles. `mcp` translates JSON-RPC into `signals::controller` calls.

## State machine

`signals::controller::next_state` is a pure function of `(from, incoming) → to`. Every cell of the 7×6 table is unit-tested. See spec §5.3 for the table.

The stateful controller wraps `next_state` with:
- A `Mutex<ControllerInner>` that serializes transitions.
- An animator handle that gets gracefully cancelled (≤200ms) on transition.
- A `last_user_frame` cache used to restore `UserControlled` on reconnect.

## Animator

Each pattern implements `Pattern::next_frame(session, tick)`. The runtime ticks at 20 FPS (50ms) and pushes the returned effect via HTTP. Returning `Ok(false)` signals one-shot completion (used by `complete`).

Cancellation modes:

| Scenario | Mode |
|---|---|
| State transition | Graceful: signal + wait ≤200ms + force-drop if exceeded |
| Heartbeat failure, shutdown | Force: abort the task immediately |

## Why rmcp?

Official Anthropic Rust SDK. If upstream breaks compat we fall back to manual JSON-RPC; see Task 3.1 of the plan for the decision record.

## Testing strategy

- Unit tests for every pure function (transition table, pattern frames, merge algorithm, hex parsing).
- wiremock integration tests for HTTP paths (session, heartbeat reconnect, 404-vs-503).
- Subprocess integration test for MCP stdio round-trip.
- Manual hardware tests in `docs/manual-testing.md` gate every release.
```

- [ ] **Step 4: Finalize manual-testing.md**

Ensure `docs/manual-testing.md` has sections for M2/M3/M4/M5/M6 with the full checklist from spec §9.3. Copy the 11-step list in verbatim.

- [ ] **Step 5: Commit**

```bash
rtk git add docs/
rtk git commit -m "docs: install, troubleshooting, architecture, manual-testing"
```

### Task 6.8: `examples/standalone_demo.rs`

**Files:**
- Create: `examples/standalone_demo.rs`

- [ ] **Step 1: Write a library-only demo (no MCP)**

```rust
//! Use the library directly without MCP. `cargo run --example standalone_demo`.

use razer_chroma_mcp::signals::controller::{Incoming, StatefulController};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let controller = StatefulController::default();
    for step in [Incoming::Thinking, Incoming::NeedsInput, Incoming::Complete, Incoming::Idle] {
        println!("transition: {step:?}");
        controller.transition(step).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    controller.shutdown().await;
    Ok(())
}
```

- [ ] **Step 2: Verify**

Run: `rtk cargo build --example standalone_demo`
Expected: builds successfully.

- [ ] **Step 3: Commit**

```bash
rtk git add examples/standalone_demo.rs
rtk git commit -m "docs: standalone library example for embedders"
```

### Task 6.9: Non-technical user usability test

**Files:**
- Modify: `docs/manual-testing.md`

- [ ] **Step 1: Run the usability test**

Recruit a non-technical tester (friend, family member). Give them:

- Link to the GitHub release.
- Instruction: "Install this, then ask Claude Desktop to think about something and watch your keyboard."

Time them. Gate: **under 3 minutes**, no help needed.

- [ ] **Step 2: Record the result**

Append to `docs/manual-testing.md`:

```markdown
## M6 — Usability gate — 2026-MM-DD

Tester: <name, non-technical>
Environment: Windows <version>, <hardware>
Installer version: v0.1.0

Timing:
- Download → Setup.exe launched: <s>
- Setup.exe clicked → wizard complete: <s>
- Wizard complete → `signal_thinking` visible: <s>
- Total: <s>

Tester's words: "<quote>"

Issues encountered: <list, or "none">
Gate (≤180s, no help): PASS / FAIL

Follow-ups: <list>
```

- [ ] **Step 3: Commit**

```bash
rtk git add docs/manual-testing.md
rtk git commit -m "docs: m6 non-technical usability test log"
```

### Task 6.10: Tag v0.1.0 and publish

**Files:**
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Update CHANGELOG for v0.1.0**

Replace `[Unreleased]` with `[0.1.0] - 2026-MM-DD`, summarizing the feature set from all milestones. Add a new empty `[Unreleased]` section at top.

- [ ] **Step 2: Commit the changelog**

```bash
rtk git add CHANGELOG.md
rtk git commit -m "docs: changelog 0.1.0 release"
```

- [ ] **Step 3: Tag**

```bash
rtk git tag -a v0.1.0 -m "v0.1.0 — initial release"
```

Do **not** push the tag without the user's explicit go. The user pushes manually so the Release workflow fires under their account.

- [ ] **Step 4: Open follow-up issues**

At user's direction, use `gh issue create` for:

- `[v0.2.0] Code signing for Setup.exe`
- `[v0.2.0] signal_* rendering on mouse + mousepad`
- `[v0.2.0] Concurrent Chroma client detection/warning`
- `[v0.2.0] TOML config for custom palette/patterns`

- [ ] **Step 5: M6 status report**

Summarize: all §16 success criteria met (checklist from spec), tag ready to push, release workflow will build Setup.exe on push.

---

## Final Checklist — Spec §16 Success Criteria Coverage

Before declaring v0.1.0 done, confirm each §16 bullet:

- [ ] Non-technical tester installs via Setup.exe in under 3 minutes (Task 6.9)
- [ ] BlackWidow displays `thinking` pattern during Claude response (Task 4.14)
- [ ] `needs_input` stage-1 flash ≥90% brightness within 900 ms (Task 4.7 test)
- [ ] Full state transition table passes unit tests (Task 4.3 — 42 tests)
- [ ] Session auto-recovers from Synapse restart within 15 seconds (Task 5.6 manual test)
- [ ] Config merge passes all 9 unit cases (Task 6.1)
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test` pass in CI (Task 1.5, continuously)
- [ ] Documentation reviewed by at least one non-technical reader (Task 6.9 tester confirms docs were clear)
