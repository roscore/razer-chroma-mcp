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
  ; Call our own uninstall subcommand to clean Claude configs first.
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
