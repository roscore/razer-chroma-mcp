# razer-chroma-mcp

> Give Claude a body language on your Razer keyboard.

When Claude is thinking, your BlackWidow pulses terracotta. When Claude needs
your input, the keyboard unmistakably calls for you. When Claude finishes, a
Claude-branded asterisk flashes once and rests. All in Anthropic's warm palette.

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

The 5 `signal_*` patterns render on any Razer keyboard using the generic
6×22 grid. Mouse, mousepad, headset, keypad, and Chroma Link work with the
low-level tools (`set_static_color` etc.) but do not render signals in
v0.1.0 — see `[v0.2.0] signal_* rendering on mouse + mousepad` in Issues.

## Not using Claude Desktop? `cargo install`

```powershell
cargo install razer-chroma-mcp
razer-chroma-mcp setup
```

## Troubleshooting

- Keyboard flickers or fights another app → [last-writer-wins explainer](docs/troubleshooting.md#my-keyboard-flickers--another-app-is-controlling-it)
- SmartScreen "Unrecognized app" on install → [install guide](docs/install.md#smartscreen-warning)
- Nothing happens → run `razer-chroma-mcp doctor`

## Docs

- [Install guide with screenshots](docs/install.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Architecture (for contributors)](docs/architecture.md)
- [Manual testing checklist](docs/manual-testing.md)

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

## Trademarks

Razer is a trademark of Razer Inc. This project is not affiliated with or
endorsed by Razer.

Anthropic and Claude are trademarks of Anthropic, PBC. Warm palette used
for aesthetic fit; this project is not affiliated with or endorsed by
Anthropic.

---

## 한국어 요약

Claude Desktop / Claude Code가 Razer Chroma 키보드를 통해 **생각 중 / 입력 대기 / 완료 / 오류**
상태를 시각적으로 표현하도록 해 주는 MCP 서버입니다. 비개발자도 Setup.exe를
더블클릭해 3분 안에 설치할 수 있습니다. 터미널이 필요 없습니다.
