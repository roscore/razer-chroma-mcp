# Troubleshooting

## Check first: `razer-chroma-mcp doctor`

Run from a terminal (or via Start Menu → Razer Chroma MCP Setup → wait
for wizard → it runs doctor at the end). Each `[x]` item prints a fix.

## Symptoms

### Nothing happens when I ask Claude a question

- Is the server registered? Check
  `%APPDATA%\Claude\claude_desktop_config.json` contains `"razer-chroma"`
  under `"mcpServers"`.
- Did you restart Claude Desktop after setup? MCP servers load at
  startup only.
- Run `razer-chroma-mcp doctor`.

### My keyboard flickers / another app is controlling it

Razer's Chroma SDK explicitly allows multiple apps to drive the keyboard
at once — whichever writes the last frame wins. If you have another
Chroma-aware app running (a game, another RGB utility), they will fight
for the keyboard. Close the other app, or accept flicker.

### SmartScreen blocks the installer

v0.1.0 is unsigned. Click **More info → Run anyway**. v0.2.0 will be
signed.

### Synapse is running but `doctor` says "Chroma REST API unreachable"

Open Synapse → **Chroma** → ensure **Chroma Apps** toggle is on. The
REST API listens on `http://localhost:54235` only when this is enabled.

### Where are the logs?

`%LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log`. Rotated daily, retained
7 days.

### I use Claude Code, not Claude Desktop

The wizard will detect `%USERPROFILE%\.claude.json` and offer to register
there too.

## Reset

Uninstall, then delete `%APPDATA%\Claude\claude_desktop_config.json.bak-*`
and `%USERPROFILE%\.claude.json.bak-*` if you want to remove history.
