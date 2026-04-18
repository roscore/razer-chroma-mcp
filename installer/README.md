# Building the Windows installer

Requires NSIS 3.x (`choco install nsis` or <https://nsis.sourceforge.io/>) and
the `EnVar` plugin (copy `EnVar.dll` into NSIS `Plugins\x86-ansi\`).

## Steps

1. Build the release binary from the repo root:

   ```powershell
   cargo build --release
   ```

2. Compile the installer from this directory:

   ```powershell
   makensis /DVERSION=0.1.0 razer-chroma-mcp.nsi
   ```

3. Output: `razer-chroma-mcp-setup-0.1.0.exe`

CI builds this automatically on tag push; see `.github/workflows/release.yml`.

## Design notes

- **Per-user install** — `RequestExecutionLevel user` + `InstallDir` under
  `%LOCALAPPDATA%\Programs\razer-chroma-mcp`. No UAC prompt, no admin
  rights, no system-wide side effects.
- **`EnVar`** adds `$INSTDIR` to the user's `PATH` (HKCU) so
  `razer-chroma-mcp doctor` / `setup` / `pattern` work from any terminal
  after install.
- **Finish-page auto-launch** runs `razer-chroma-mcp.exe setup` so the
  user is dropped straight into the wizard.
- **Uninstaller** invokes `razer-chroma-mcp.exe uninstall` first to scrub
  the `razer-chroma` entry from Claude Desktop / Code configs before
  deleting files.
