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
