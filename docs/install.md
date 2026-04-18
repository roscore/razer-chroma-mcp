# Install guide

## 1. Download

Get `razer-chroma-mcp-setup-<version>.exe` from the
[Releases page](https://github.com/roscore/razer-chroma-mcp/releases).

## 2. Run the installer

Double-click the downloaded file.

### SmartScreen warning

Windows SmartScreen will show **"Windows protected your PC"** because
v0.1.0 is unsigned (code-signing certificates cost money; we're deferring
it to v0.2.0). To continue:

1. Click **More info**.
2. Click **Run anyway**.

![SmartScreen screenshot placeholder — add at release](../installer/assets/smartscreen.png)

(If you don't trust the installer, verify the SHA-256 hash published on
the release page, or install via `cargo install razer-chroma-mcp`.)

## 3. Follow the wizard

The wizard will:

1. Check that Razer Synapse is running.
2. Play a 3-second terracotta sweep on your keyboard to verify the
   Chroma SDK.
3. Ask whether to register with Claude Desktop and/or Claude Code.
4. Run diagnostics.

## 4. Restart Claude Desktop

Close and re-open Claude Desktop. Ask Claude to think about something —
the keyboard should pulse terracotta.

## Uninstall

Go to **Settings → Apps → Installed apps**, find **razer-chroma-mcp**,
click **Uninstall**. The uninstaller will remove our entry from Claude
Desktop/Code configs (backups kept as `*.bak-<timestamp>`).
