# Razer Chroma MCP Server — Design Spec

- **Date**: 2026-04-17
- **Author**: roscore
- **Status**: Draft for review (revision 2 — post-review)
- **Target version**: v0.1.0
- **License**: MIT OR Apache-2.0
- **Repo**: https://github.com/roscore/razer-chroma-mcp

## 1. Purpose & Differentiators

A Rust MCP server that lets Claude Desktop / Claude Code control Razer Chroma RGB devices through Razer Synapse's local REST API.

The project is **not** a generic "RGB control MCP". Its core value proposition is:

> **Give Claude a body language on your keyboard.** When Claude is thinking, the BlackWidow softly pulses. When Claude needs your input, the keyboard unmistakably calls for you. When Claude finishes, a Claude-branded starburst flashes once and rests. All in Anthropic's warm palette.

Non-goals:

- Linux support (use `openrazer` instead — mention in README)
- Game integrations (this is about Claude, not Fortnite)
- User-facing GUI (CLI + setup wizard only; the "UI" is the keyboard itself)
- Profile management (Synapse already does this)

## 2. Target Users

Two audiences, both first-class:

1. **Non-technical Claude users** — install via double-click, never open a terminal.
2. **Rust developers** — `cargo install`, hack on patterns, extend.

Installation UX is designed primarily for audience 1. Documentation serves both.

## 3. Architecture Overview

```
┌─────────────────┐   stdio JSON-RPC   ┌──────────────────────────────┐
│ Claude Desktop  │ ◀──────────────▶ │  razer-chroma-mcp (binary)    │
│ / Claude Code   │                    │                               │
└─────────────────┘                    │  ┌────────────────────────┐  │
                                        │  │ mcp::server (rmcp)     │  │
                                        │  │  - tool registry       │  │
                                        │  │  - input validation    │  │
                                        │  └────────────┬───────────┘  │
                                        │               │              │
                                        │  ┌────────────▼───────────┐  │
                                        │  │ signals::controller    │  │
                                        │  │  - current ClaudeState │  │
                                        │  │  - animator lifecycle  │  │
                                        │  └────────────┬───────────┘  │
                                        │               │              │
                                        │  ┌────────────▼───────────┐  │
                                        │  │ chroma::session        │  │
                                        │  │  - init/heartbeat/drop │  │
                                        │  │  - reconnect backoff   │  │
                                        │  └────────────┬───────────┘  │
                                        │               │              │
                                        └───────────────┼──────────────┘
                                                        │ HTTP REST
                                                        ▼
                                         http://localhost:54235 (Synapse)
```

**Key design principles:**

- Three layers (`mcp` / `signals` / `chroma`) with strict one-way dependencies: `mcp → signals → chroma`.
- All animation happens in `signals`; `chroma` only knows about single effect payloads.
- `signals::controller` is the only owner of "current keyboard state" — both the low-level MCP tools and the high-level `signal_*` tools go through it, so they can never fight.
- Concurrency model: tokio multi-thread runtime. Long-lived tasks: (a) MCP stdio loop, (b) heartbeat loop, (c) current animator (spawned/cancelled on state change).
- **Concurrent Chroma client policy (last-writer-wins):** Chroma SDK explicitly permits multiple concurrent clients, and effects are non-cooperative — whoever writes a frame last wins. This server does **not** attempt to coordinate with Synapse profiles, games, or other SDK clients. If another process is actively pushing frames, our effects may appear flickery or be overwritten. We document this in README troubleshooting; we do not detect it programmatically in v0.1.0 (deferred).
- **Startup behavior:** The server initializes the Chroma session on first MCP tool call, *not* eagerly on process start. This minimizes friction when Claude Desktop loads the server but the user isn't actively using it, and avoids claiming the keyboard from Synapse unnecessarily. The `demo` and `doctor` commands initialize eagerly since they're explicit user actions.
- **Heartbeat cadence:** 5-second interval, 2-second HTTP timeout per heartbeat, 3 consecutive failures triggers reconnect.

## 4. Module Layout

```
src/
├── main.rs                      # clap entrypoint, tracing init, log file path
├── lib.rs                       # re-exports public API
├── error.rs                     # ChromaError (thiserror); anyhow at binary boundary
├── cli/
│   ├── mod.rs                   # clap::Parser definitions
│   ├── serve.rs                 # default — runs MCP stdio server
│   ├── setup.rs                 # install wizard (Claude Desktop/Code detection + config write)
│   ├── doctor.rs                # environment diagnostics
│   ├── demo.rs                  # 3-second terracotta sweep for hardware validation
│   ├── pattern.rs               # preview any named pattern without Claude Desktop (N3)
│   └── uninstall.rs             # removes Claude config entries cleanly
├── config/
│   ├── mod.rs
│   ├── merge.rs                 # JSON deep-merge for Claude Desktop config (see §7.6)
│   └── paths.rs                 # platform config path resolution (Claude Desktop/Code)
├── chroma/
│   ├── mod.rs
│   ├── client.rs                # reqwest wrapper, 2s default timeout
│   ├── session.rs               # init/heartbeat/graceful drop, reconnect
│   ├── effects.rs               # Effect* types + serde payloads (NONE/STATIC/CUSTOM/WAVE/SPECTRUM/BREATHING/REACTIVE)
│   ├── devices.rs               # Device enum, grid constants (KEYBOARD_ROWS=6, KEYBOARD_COLS=22, etc.)
│   └── color.rs                 # Rgb type, Hex parser, RGB ↔ BGR u32 conversion
├── signals/
│   ├── mod.rs                   # ClaudeState enum
│   ├── controller.rs            # state machine, animator cancel/spawn, last-state restoration
│   ├── palette.rs               # Claude palette constants
│   ├── inactivity.rs            # idle auto-entry timer (N2)
│   └── patterns/
│       ├── mod.rs
│       ├── thinking.rs          # radial pulse frame generator
│       ├── needs_input.rs       # 3-stage animation (flash → breathe + counter-rhythm accent)
│       ├── idle.rs              # deep ember slow breathing
│       ├── complete.rs          # sweep + asterisk one-shot
│       └── error.rs             # burnt rust slow breathing
└── mcp/
    ├── mod.rs
    ├── server.rs                # rmcp server bootstrap
    └── tools.rs                 # 12 tool handlers
```

## 5. Claude State Signaling

### 5.1 Palette (Anthropic warm family only)

| Name | Hex | Usage |
|---|---|---|
| Terracotta | `#D97757` | Primary brand accent — thinking, complete, needs_input base |
| Warm Cream | `#F0EEE6` | Bone/light accent — complete asterisk flash, needs_input accent keys |
| Deep Ember | `#3E2723` | Idle base — quiet fireside |
| Burnt Rust | `#8D3A1C` | Error — terracotta cooled down, not aggressive red |

No cyan, mint, or pure red. All transitions stay within this family.

### 5.2 States

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaudeState {
    /// Keyboard is not being driven by us; Synapse default profile is visible.
    /// Entered on startup, on graceful shutdown, and on explicit `clear_all`.
    Off,
    /// We are driving the keyboard, but a low-level tool (user-controlled frame)
    /// set the current frame — we hold it until the next transition.
    UserControlled,
    Idle,
    Thinking,
    NeedsInput,
    /// One-shot transition state. On completion, restores the previous persistent state.
    /// Invariant: `previous` is always a `PersistentState`, which excludes `Complete` —
    /// `Complete` is never nested inside itself. Enforced by the type.
    Complete { previous: PersistentState },
    Error,
}

/// The states that `Complete` can restore to. Excludes `Complete` itself by construction.
/// Kept as its own enum (rather than `ClaudeState`) to make that invariant type-level
/// rather than a runtime check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersistentState { Off, UserControlled, Idle, Thinking, NeedsInput, Error }
```

`PersistentState` is itself `Copy` and small (single byte discriminant), so `ClaudeState` remains `Copy`.

### 5.3 State Transition Table

Handlers are serialized through a single `Mutex<ControllerInner>`. A transition always: (a) requests the current animator to cancel, (b) awaits its exit with a 200 ms bound (force-drops if exceeded — see §5.5), (c) spawns the new animator. Signal handlers thus naturally enforce last-write-wins without queuing.

| From \ Incoming | `signal_thinking` | `signal_needs_input` | `signal_idle` | `signal_complete` | `signal_error` | *low-level tool call* |
|---|---|---|---|---|---|---|
| `Off` | → Thinking | → NeedsInput | → Idle | → Complete{previous:Off} | → Error | → UserControlled |
| `UserControlled` | → Thinking | → NeedsInput | → Idle | → Complete{previous:UserControlled} | → Error | → UserControlled (frame replaced) |
| `Idle` | → Thinking | → NeedsInput | (no-op) | → Complete{previous:Idle} | → Error | → UserControlled |
| `Thinking` | (no-op, restart animator) | → NeedsInput | → Idle | → Complete{previous:Thinking} | → Error | → UserControlled |
| `NeedsInput` | → Thinking | (no-op, restart animator) | → Idle | → Complete{previous:NeedsInput} | → Error | → UserControlled |
| `Complete` | → Thinking (interrupts one-shot) | → NeedsInput (interrupts) | → Idle (interrupts) | (no-op, restart) | → Error (interrupts) | → UserControlled (interrupts) |
| `Error` | → Thinking | → NeedsInput | → Idle | → Complete{previous:Error} | (no-op, restart) | → UserControlled |

Complete auto-transition: when the Complete one-shot finishes, the controller itself issues a synthetic transition to `previous`.

### 5.4 Pattern Specifications

| State | Pattern | Color | Speed | Scope | Impl |
|---|---|---|---|---|---|
| `thinking` | Radial pulse from keyboard center outward | Terracotta → Cream gradient | 2.8s cycle | Full keyboard, 35–65% brightness | Self-animator, `CHROMA_CUSTOM` @ 20 FPS |
| `needs_input` | **3-stage, high-impact**: (1) Triple flash at 0.3s intervals (full keyboard cream ≥90% brightness, each flash ≤300 ms, total stage ≤900 ms) → (2) Full keyboard terracotta breathing, 1.2s cycle, 60–100% → (3) Spacebar + Enter + Esc in cream with **counter-rhythm** 0.75s pulse (intentional polyrhythm against the 1.2s base — do not "fix" the mismatch) | Terracotta + Cream | Loops until state changes | Full keyboard + accent keys | Self-animator, `CHROMA_CUSTOM` @ 20 FPS |
| `idle` | Very slow breathing | Deep Ember | 12s cycle | Full keyboard, 5–12% brightness | `CHROMA_BREATHING` built-in (single HTTP call) |
| `complete` | Left→right terracotta sweep → central cream **asterisk** flash → fade to previous state | Terracotta + Cream | 1.8s one-shot | Full keyboard, peak 70% | Self-animator, `CHROMA_CUSTOM` @ 20 FPS |
| `error` | Slow deep breathing | Burnt Rust | 2.5s cycle | Full keyboard, 40–80% | `CHROMA_BREATHING` built-in |

### 5.5 Asterisk Shape (for `complete`)

Claude's brand mark is a geometric starburst. We approximate it on the 6×22 grid as 8 rays from the center (approx col 11, between rows 2–3):

```
Row 0:  . . . . . . . . . . * . . . . . . . . . . .
Row 1:  . . . . * . . . . . * . . . . . * . . . . .
Row 2:  . . . . . * . . . * * * . . . * . . . . . .
Row 3:  . . . . . . * . * * * * * * . . . . . . . .
Row 4:  . . . . . * . . . * * * . . . * . . . . . .
Row 5:  . . . . * . . . . . * . . . . . * . . . . .
```

Held for 180 ms cream at 85% brightness, then fades over 400 ms.

### 5.6 Device scope for v0.1.0

Only the **generic 6×22 keyboard grid** (`/keyboard`) is targeted for patterns. This covers BlackWidow and all Chroma keyboards using the generic `Custom` payload. Per-device variants (`CustomKey2`, etc.) and non-keyboard devices (mouse, mousepad) receive only the low-level tools (`set_static_color`, etc.), not `signal_*` patterns. A TODO is filed for v0.2.0 to extend `signal_*` rendering to mouse + mousepad.

### 5.7 Animator Contract

- Implemented as a tokio task that takes a `CancellationToken` and a shared `ChromaSession` reference.
- Generates frames at 20 FPS (50 ms tick), sends each as `CHROMA_CUSTOM` via `PUT /keyboard`.
- Each HTTP send uses an **abortable** `reqwest` request: on cancel, the in-flight request is dropped (`tokio::select!` with cancel), not awaited.
- Cancellation modes:
  - **Graceful cancel** (state transition): controller requests cancel, awaits animator exit up to 200 ms. If the animator is stuck (e.g. Synapse unresponsive), the controller force-drops the task handle and spawns the new animator regardless.
  - **Force cancel** (heartbeat failure / shutdown): controller immediately drops the task handle without awaiting. The animator's in-flight request is cancelled by drop.
- Controller invariants:
  - Exactly one animator exists per device at any time.
  - Animator never holds the `ControllerInner` mutex across an HTTP call (it snapshots any needed data then releases).

### 5.8 Interaction with Low-Level MCP Tools

- Any low-level call (`set_static_color`, `set_custom_grid`, etc.) implicitly transitions state to `UserControlled` and cancels the current animator. The keyboard holds the user-set frame until the next transition.
- Any `signal_*` call transitions to the corresponding state per §5.3.
- On heartbeat reconnect, controller re-applies the last persistent state automatically — except `UserControlled`, which is re-applied only if the last user frame was cached (we cache the last `set_custom_grid` / `set_static_color` payload for this purpose).

### 5.9 Idle Auto-Entry

`signals::inactivity` tracks the timestamp of the most recent `signal_*` or low-level tool call. If the server has been in `UserControlled` or `Error` for more than **10 minutes** with no activity, the controller auto-transitions to `Idle`.

**Explicitly excluded from auto-entry**: `Thinking` and `NeedsInput`. Both are semantically "someone is waiting" — `Thinking` because Claude is mid-work (a long response may legitimately not refresh the signal), `NeedsInput` because the entire purpose is to keep signaling until the user responds. Auto-demoting either would defeat the feature.

The timeout is hard-coded for v0.1.0; overrideable via env `RAZER_CHROMA_IDLE_MS` for testing.

## 6. MCP Tools (12 total)

### 6.1 Low-level (7)

| Tool | Args | Returns |
|---|---|---|
| `set_static_color` | `device: "all"\|"keyboard"\|...`, `color: "#RRGGBB"\|{r,g,b}` | `{status}` |
| `set_custom_grid` | `device`, `grid: [[color, ...], ...]` | `{status}` |
| `play_preset_effect` | `device`, `effect_type: "WAVE"\|"SPECTRUMCYCLING"\|"BREATHING"\|"REACTIVE"`, `params: object` | `{status}` |
| `clear_device` | `device` | `{status}` |
| `clear_all` | (none) | `{status}` |
| `get_device_info` | (none) | `{devices: [{name, rows, cols}, ...]}` |
| `get_session_status` | (none) | `{initialized, last_heartbeat, state, failures}` |

### 6.2 High-level Claude signals (5)

| Tool | Args | Behavior |
|---|---|---|
| `signal_thinking` | (none) | Transition → Thinking |
| `signal_needs_input` | (none) | Transition → NeedsInput (high-impact 3-stage animation) |
| `signal_idle` | (none) | Transition → Idle |
| `signal_complete` | (none) | Play Complete one-shot, return to previous persistent state (§5.3) |
| `signal_error` | `message?: string` | Transition → Error (message logged, not displayed) |

All inputs JSON-Schema validated. Invalid inputs return MCP errors with actionable messages.

## 7. Installation UX

### 7.1 Distribution

- **Primary**: Windows NSIS installer (`razer-chroma-mcp-setup-x.y.z.exe`) from GitHub Releases.
- Installer behavior:
  - **Per-user install, no UAC elevation required.** All writes to `%LOCALAPPDATA%` and `HKCU` only.
  - Installs binary to `%LOCALAPPDATA%\Programs\razer-chroma-mcp\`.
  - Adds to **user** `PATH` via `HKCU\Environment`.
  - Creates Start Menu shortcut ("Razer Chroma MCP Setup").
  - Registers in Windows "Apps & features" via `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\`.
  - Optional checkbox: "Launch setup wizard now" (default on).
- **Code signing**: Unsigned for v0.1.0 (Authenticode cert is a paid dependency deferred to v0.2.0). SmartScreen will show "Unrecognized app" warning on first launch; README `install.md` documents the "More info → Run anyway" flow with a screenshot. NSIS installers also trigger AV false positives occasionally; we minimize surface area by avoiding scripted sections and submit to Microsoft SmartScreen reputation as the install count grows. A tracking issue (`[v0.2.0] Code signing`) will be created at release.
- **Secondary**: `cargo install razer-chroma-mcp` for Rust developers. Documented as alternative; does not perform setup wizard automatically (user runs `razer-chroma-mcp setup` manually).

### 7.2 Setup Wizard (`razer-chroma-mcp setup`)

Runs automatically post-install; also callable manually. Flow:

1. **Detect Synapse.** Check if process is running + HTTP probe to `http://localhost:54235/`. If missing, open browser to `https://www.razer.com/synapse` and wait for user Enter.
2. **Verify Chroma SDK.** Attempt a minimal session init. On failure, distinguish "Synapse running but Chroma module off" vs other issues and guide accordingly.
3. **Hardware confirmation.** Play `demo` pattern (3-second terracotta sweep on keyboard) and ask "Did you see the lights move? [Y/n]".
4. **Detect Claude Desktop.** Scan `%APPDATA%\Claude\claude_desktop_config.json`. If found, prompt "Register razer-chroma-mcp with Claude Desktop? [Y/n]". On yes: apply the §7.6 merge strategy.
5. **Detect Claude Code.** Scan `%USERPROFILE%\.claude.json`. Same flow.
6. **Run doctor.** Print final status summary.
7. **Instruct user**: "Restart Claude Desktop to activate."

Prompts use a **simple bilingual heuristic**: `GetUserDefaultUILanguage() == 0x0412 (ko-KR)` → Korean line first, English line second. Otherwise English first, Korean second. User can force a language via `--lang ko|en`.

### 7.3 `doctor` Command

Checks, in order:

- [ ] Razer Synapse installed (registry check)
- [ ] Razer Synapse running (process check)
- [ ] Chroma REST API reachable (`GET http://localhost:54235/`)
- [ ] Chroma session init succeeds
- [ ] At least one Chroma-compatible device detected
- [ ] Claude Desktop config present and contains our entry
- [ ] Claude Code config present and contains our entry (optional)

Output uses color + icons: ✅/❌/⚠️. Each failed item prints "How to fix". `doctor` also prints the log file path (§12).

### 7.4 `demo` & `pattern` Commands

- `demo`: runs session init → plays 3-second terracotta→cream left-to-right sweep on keyboard → session teardown. Returns 0 on success, non-zero with reason on failure.
- `pattern <name>`: previews any named pattern (`thinking`, `needs_input`, `idle`, `complete`, `error`) for 8 seconds (or until Ctrl+C) without needing Claude Desktop. Added at reviewer N3 suggestion so non-devs can verify each pattern looks right.

### 7.5 `uninstall` Command

Called by NSIS uninstaller. Behavior:

- Detects whether Claude Desktop is currently running. If yes, prompts "Please close Claude Desktop before uninstalling to avoid config corruption. Continue anyway? [y/N]".
- Reads each config file, parses JSON, removes only the `mcpServers.razer-chroma` entry **by key match** (does not touch other entries regardless of shape).
- Creates a `.bak-<timestamp>` backup before writing.
- Idempotent: if our entry is not present, no-op success.
- Always uses atomic write (§7.6).

### 7.6 Claude Desktop / Claude Code config merge strategy

This is the single riskiest operation in the installer and has its own specification.

**Inputs**: target config path (JSON), our server entry.

**Our entry shape** (for Claude Desktop):
```json
{
  "mcpServers": {
    "razer-chroma": {
      "command": "<absolute path to razer-chroma-mcp.exe>",
      "args": ["serve"]
    }
  }
}
```

**Algorithm**:

1. **Read** the file. If missing, treat as `{}`.
2. **Parse** as strict JSON (no comments, no trailing commas — Claude Desktop writes strict JSON; any user hand-edit with JSONC syntax is treated as parse failure). If parse fails:
   - Copy the file to `<path>.corrupt-<timestamp>` for user recovery.
   - Abort with a clear error telling the user where the quarantine is and that they must fix or delete the corrupt file.
3. **Validate** root is an object; if not, abort with error (do not overwrite non-standard configs).
4. **Deep-merge** our entry:
   - If `mcpServers` key is absent → create it as an object with our entry.
   - If `mcpServers` is an object → add/replace our `razer-chroma` key, preserving all other keys in the object.
   - If `mcpServers` is present but **not an object** (e.g. array, string, number, null) → abort with an error analogous to step 3; do not clobber. The error directs the user to manually fix the file or delete it.
   - Do not touch any other top-level keys.
5. **Atomic write**: serialize to a temp file in the same directory (`<path>.tmp-<pid>`), `fsync`, then `rename` over the original. On Windows, use `MoveFileEx` with `MOVEFILE_REPLACE_EXISTING`.
6. **Backup**: before step 5, copy the original to `<path>.bak-<timestamp>`. Retain the 3 most recent backups; prune older.
7. **Idempotent re-run**: if the current entry is byte-for-byte identical to ours, skip write.
8. **Race with Claude Desktop**: we do not lock the file (Windows has no portable advisory file locks that Claude Desktop also respects). The atomic rename means Claude Desktop either sees the old file or the new file — never a partial write. If Claude Desktop wrote after our read but before our rename, we may overwrite its changes; this is acceptable because Claude Desktop writes this file only at startup/config-UI-save moments, and we prompt the user to close Claude Desktop before setup. Uninstall warns even louder (§7.5).

Unit tests cover: missing file, empty file, `{}`, file with other servers, malformed JSON, non-object root, **non-object `mcpServers`**, idempotent re-run, backup rotation.

## 8. Error Handling

### 8.1 Error Type

```rust
#[derive(thiserror::Error, Debug)]
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
    InvalidGridSize { expected: (usize, usize), actual: (usize, usize) },
    #[error("Invalid color: {0}")]
    InvalidColor(String),
    #[error("Unknown device: {0}")]
    UnknownDevice(String),
    #[error("Config file corrupt: {path}. A copy has been saved to {quarantine}.")]
    ConfigCorrupt { path: String, quarantine: String },
}
```

### 8.2 Recovery Behavior

| Condition | Action |
|---|---|
| Session init HTTP connection refused | `SynapseNotRunning` |
| Session init returns 404/503 on specific path | Probe root; distinguish Synapse-off vs Chroma-off |
| Heartbeat fails 1–2× | Exponential backoff retry (500ms → 1s → 2s); state stays healthy |
| Heartbeat fails 3× consecutively | Save current persistent state, **force-cancel** current animator (§5.7), re-init session, restore state (including last user frame if `UserControlled`) |
| Re-init fails 5× consecutively | Transition to `SessionStatus::Degraded`; MCP tool calls return errors with guidance; heartbeat continues to retry every 30s |
| Ctrl+C / SIGTERM | Graceful: cancel animator → `CHROMA_NONE` → `DELETE session` → exit 0 |
| stdio EOF (client closed) | Same graceful shutdown |
| Config file parse failure | Copy to `.corrupt-<ts>`, abort with `ConfigCorrupt` error |

All HTTP calls have a 2-second default timeout (configurable via env var `RAZER_CHROMA_TIMEOUT_MS`).

## 9. Testing Strategy

### 9.1 Unit Tests (target 80%+ coverage)

- `chroma::color`: Hex parsing (`#FFF`, `#FFFFFF`, case, bad inputs), RGB ↔ BGR u32 round-trip.
- `chroma::effects`: Each effect payload serializes to a golden JSON snapshot (using `insta` crate, per reviewer N5).
- `chroma::devices`: Grid size constants match expected device specs.
- `signals::patterns::*`: First/middle/last frame of each pattern matches expected colors at expected keys. For `needs_input`, verify (a) stage 1 produces ≥3 frames with all keys ≥230/255 cream within first 900 ms, (b) stage 2 keyboard-wide brightness oscillation ≥40% amplitude, (c) stage 3 accent keys (spacebar/Enter/Esc indices) differ from baseline on counter-rhythm.
- `signals::controller`: Full state transition table (§5.3) covered — one test per (from, incoming) cell.
- `signals::inactivity`: verify auto-Idle fires after timeout with fake clock.
- `config::merge`: cases from §7.6 (missing, empty, `{}`, other servers, malformed, non-object, idempotent, backup rotation).

### 9.2 Integration Tests (`tests/integration.rs`)

Mock Chroma REST API with `wiremock`:

- Full session lifecycle: init → 3 heartbeats → effect → shutdown.
- `SynapseNotRunning` path: mock server refuses connection.
- Heartbeat failure + recovery: mock returns 503 mid-stream, animator force-cancelled cleanly.
- Degraded mode: 5 consecutive re-init failures.
- All MCP tools happy path: spawn binary as subprocess with stdio pipes, drive via rmcp client.

**Caveat**: these tests do *not* exercise real Synapse behavior (effect rendering, device enumeration correctness). Real-hardware verification is covered in §9.3 and is a **pre-release gate** — v0.1.0 is not released until §9.3 passes on a BlackWidow.

### 9.3 Manual Testing (`docs/manual-testing.md`)

Live hardware checklist (BlackWidow):

1. `razer-chroma-mcp doctor` → all ✅
2. `razer-chroma-mcp demo` → terracotta sweep 3 seconds
3. `razer-chroma-mcp pattern needs_input` → observe 3-stage animation
4. Claude Desktop: ask Claude to think about something → thinking pattern (radial pulse)
5. Claude Desktop: request requiring approval → needs_input animation (verified measurably per §5.4)
6. Completion → complete sweep + asterisk
7. Trigger error → error pattern (burnt rust breathing)
8. Idle for 10+ minutes without activity → auto-transition to idle (§5.9)
9. Force-quit Claude Desktop → keyboard returns to Synapse default profile
10. Force-quit Synapse mid-session → doctor surfaces issue; restart Synapse → automatic recovery within 15 seconds
11. Install flow end-to-end: Setup.exe → wizard → Claude Desktop registered → restart Claude Desktop → `signal_thinking` visible

Usability gate: one non-technical tester completes install + step 11 in under 3 minutes without asking for help.

## 10. CI/CD

GitHub Actions workflow on `windows-latest`:

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo build --release`

On tag push (`v*`):

- Build Setup.exe via NSIS (`makensis` available on windows-latest runners).
- Upload Setup.exe + `cargo`-built binary to GitHub Releases.

Additional: `cargo build` only (no release, no NSIS) on `macos-latest` and `ubuntu-latest` to catch portability issues, but **not distributed**.

## 11. Documentation

- `README.md`: Project intro, badges, 3-step install (download → double-click → done), GIF demo, supported devices, troubleshooting link. English primary, Korean secondary section.
- `docs/install.md`: ~10 screenshots from Synapse download through Claude Desktop restart, **including SmartScreen "Unrecognized app → More info → Run anyway" screenshot**.
- `docs/troubleshooting.md`: Symptom → cause → fix table. Includes "another app is driving my keyboard" entry explaining last-writer-wins semantics.
- `docs/manual-testing.md`: Checklist above.
- `docs/architecture.md`: For contributors — module map, animator contract, state transition table.
- `CONTRIBUTING.md`: Dev setup, style, PR template. Includes Razer Chroma SDK ToS summary.
- `CHANGELOG.md`: Keep-a-changelog format.
- rustdoc on all public API.
- `examples/standalone_demo.rs`: Use the library directly without MCP (for embedding).

## 12. Engineering Decisions

| Item | Decision | Rationale |
|---|---|---|
| Language edition | Rust 2021 | 2024 not yet pervasive in ecosystem |
| MSRV | **Floor 1.75**; bumps to match `rmcp` requirement when upgraded, always documented in CHANGELOG | async fn in trait stable at 1.75; upstream deps may require newer |
| Async runtime | `tokio` multi-thread | heartbeat + animator + MCP stdio concurrency |
| HTTP client | `reqwest` (rustls) | avoids native OpenSSL on Windows |
| MCP SDK | **`rmcp`** (Anthropic official); at M3 start, verify current version & API; if unworkable, fall back to manual JSON-RPC over stdio (documented decision in PR) |
| CLI parsing | `clap` derive | idiomatic, good UX defaults |
| Logging | `tracing` + `tracing-subscriber` + `tracing-appender` | structured, filterable, rotating file output |
| Log destination | `%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log`, **daily rotation, 7-day retention**. `doctor` prints the path. Stderr also used when run manually. | Non-technical users can find logs for support |
| Error | `thiserror` in lib, `anyhow` at binary | standard pattern |
| Serialization | `serde` + `serde_json` | |
| Snapshot testing | `insta` | reviewer recommendation, ergonomic |
| Animation FPS | **20 FPS (50ms tick)** | aesthetically smooth, network-reasonable |
| HTTP timeout | **2s default**, overridable via env | |
| Installer | **NSIS** (via `makensis`), **per-user, no UAC**, HKCU only | mature, free, CI-friendly on Windows |
| Code signing | **v0.1.0: none** (deferred to v0.2.0); README documents SmartScreen flow | Authenticode cert is a paid dependency |
| License | MIT OR Apache-2.0 dual | Rust convention |
| Branch model | `main` only initially | 1-person project |
| Release artifacts | **NSIS Setup.exe from v0.1.0 onward** | non-technical users are the primary audience |

## 13. Milestones

| M | Scope | Acceptance Criteria |
|---|---|---|
| **M1** Scaffolding | Cargo project, module skeleton, `clap` subcommand stubs (`serve`/`setup`/`doctor`/`demo`/`pattern`/`uninstall`), CI on windows-latest, LICENSE files, CHANGELOG | `cargo build` + CI green; `razer-chroma-mcp --help` shows all subcommands |
| **M2** Chroma client | Session init/heartbeat (5s, 2s timeout, 3-fail threshold)/drop, BGR color conversion, `CHROMA_STATIC` effect, `demo` command implemented | **Live BlackWidow: `demo` displays terracotta sweep for 3 seconds; session cleanly torn down on exit** |
| **M3** MCP server | `rmcp` stdio, `set_static_color` + `clear_all` + `get_session_status` tools, Claude Desktop can call them | Claude Desktop manual test succeeds; integration test (rmcp subprocess) passes |
| **M4** Full effects + Claude signals | All 7 low-level tools + all 5 `signal_*` tools, animator infrastructure, state controller (full §5.3 table tested), state restoration on reconnect, idle auto-entry | **Integration-test-verified**: `needs_input` stage-1 produces ≥3 cream flashes ≥90% brightness within 900 ms; full state transition table green; **plus** one logged manual observation in `docs/manual-testing.md` confirming visual impact |
| **M5** Error handling + `doctor` + `pattern` | All error variants with bilingual messages, automatic reconnect (force-cancel path), degraded mode, `doctor` subcommand full implementation, `pattern` subcommand | Kill Synapse mid-session, restart it → state auto-restored within 15s; unit tests for all error paths |
| **M6** Install UX + docs | NSIS installer script (per-user, no UAC), `setup` wizard with Claude Desktop/Code detection (§7.6 merge strategy), `uninstall` for NSIS, README + install.md (with SmartScreen screenshot) + troubleshooting.md + manual-testing.md | **Non-technical user installs via Setup.exe in under 3 minutes, never opens a terminal, and sees `thinking` pattern on BlackWidow when asking Claude a question** (live usability test, n≥1, logged in release checklist) |

Each milestone ends with a status report and explicit user approval before continuing.

## 14. Open Items / Out of Scope

**Deferred to v0.2.0:**

- User-configurable palette/patterns via TOML config (v0.1.0 patterns are hardcoded-opinionated).
- Authenticode code signing of Setup.exe.
- macOS support (Razer's macOS Chroma is in preview; revisit based on SDK maturity).
- `signal_*` pattern rendering extended to mouse + mousepad.
- Additional signal states (e.g., `signal_tool_running` with tool-specific accents).
- Concurrent Chroma client detection (warn when another process is overwriting our frames).
- GUI settings app.
- Crates.io publication (post-stabilization).

**Explicitly out of scope:**

- Linux support (point users to `openrazer`).
- Mobile devices.
- Game integrations.
- Real-time audio visualization.

## 15. Legal / Trademark

- README footer:
  - *"Razer is a trademark of Razer Inc. This project is not affiliated with or endorsed by Razer."*
  - *"Anthropic and Claude are trademarks of Anthropic, PBC. Warm palette used for aesthetic fit; this project is not affiliated with or endorsed by Anthropic."*
- Review Razer Chroma SDK terms prior to release and cite relevant clauses in CONTRIBUTING.md.

## 16. Success Criteria (for v0.1.0 release)

- [ ] Non-technical tester installs via Setup.exe in under 3 minutes without terminal use (live test, logged)
- [ ] BlackWidow displays `thinking` pattern during Claude response
- [ ] `needs_input` stage-1 flash measured ≥90% brightness within 900 ms (integration test)
- [ ] Full state transition table (§5.3) passes unit tests
- [ ] Session auto-recovers from Synapse restart within 15 seconds
- [ ] Config merge strategy (§7.6) passes all 9 unit test cases including idempotent re-run and corrupt-file quarantine
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test` all pass in CI on `windows-latest`
- [ ] Documentation reviewed for clarity by at least one non-technical reader

---

## Revision History

- **rev 1** (2026-04-17): Initial draft.
- **rev 2** (2026-04-17): Addressed review findings B1–B4, S1–S12, and adopted nice-to-haves N1, N2, N3, N4, N5. Key changes: explicit state transition table (§5.3), §5.7 animator cancellation semantics, §5.9 idle auto-entry, §7.6 config merge strategy, §7.1 per-user install / code signing stance, §12 log destination, §5.4 measurable `needs_input` criterion, Anthropic trademark line, `pattern` subcommand, `insta` for snapshots.
- **rev 2.1** (2026-04-17): Addressed second-pass review findings NS1–NS3 and NN1–NN2. Changes: §5.2 unboxed `Complete::previous` to resolve `Copy`+`Box` conflict and added invariant note; §5.9 excluded `Thinking`/`NeedsInput` from idle auto-entry; §7.6 added explicit branch for non-object `mcpServers`, strict-JSON policy note, and matching unit test case.
