# Installer facts verified on a real PC (2026-11-02)

Checked on the work PC (Windows 11) before any installer code depends on them.

## WebView2
- Runtime key: `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`, value `pv` = `156.0.4314.58` (present). The `HKCU\Software\Microsoft\EdgeUpdate\Clients\{...}` and the non-WOW6432Node `HKLM` keys had no `pv` on this PC.
- Bootstrapper: `https://go.microsoft.com/fwlink/p/?LinkId=2124703` answers 301 and then 200 with `MicrosoftEdgeWebview2Setup.exe`, 1,888,976 bytes (`Content-Type: application/octet-stream`). Final URL at test time: `https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/<guid>/MicrosoftEdgeWebview2Setup.exe` (the GUID changes, always follow the fwlink).

## MacDows NSIS package (`MacDows_0.1.0_x64-setup.exe`, per-user)
- `/S` run: exit code 0, no administrator prompt.
- Uninstall entry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\MacDows` with
  - `DisplayName` = `MacDows`
  - `DisplayVersion` = `0.1.0`
  - `InstallLocation` = `"C:\Users\fatal\AppData\Local\MacDows"` — **the value contains literal double quotes; strip them**
  - `DisplayIcon` = `"C:\Users\fatal\AppData\Local\MacDows\macdows.exe"` (quoted)
  - `UninstallString` = `"C:\Users\fatal\AppData\Local\MacDows\uninstall.exe"` (quoted)
  - `MainBinaryName` = `macdows.exe`
- Installed files: `%LOCALAPPDATA%\MacDows\macdows.exe` and `uninstall.exe`.
- Running app: starting `macdows.exe` and then running the package with `/S` finished in 0.7 s with exit code 0 and the package itself ended the running `macdows.exe`. The installer therefore does not need its own "end running app" step; it only needs to report the exit code.
- Uninstall: `uninstall.exe /S` exit 0; the uninstall key and the files were gone within 1 s; `%APPDATA%\MacDows\settings.json` was kept. `HKCU\Software\macdows` (the NSIS install-dir key) stays behind (harmless).

## Not verified
A PC without WebView2, no internet, upgrade over an older version, SmartScreen behavior.
