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

## End-to-end run of the built installer (2026-11-02, this PC, Windows 11, Turkish UI)
- `node scripts/build-installer.mjs` built the app and the installer; `installer/dist/MacDows-Setup.exe` is **5.34 MB** (target 15 MB). The embedded NSIS package was built after the logo change, so the installed `macdows.exe` carries the new icon (checked by extracting the exe icon).
- Real window opened in Turkish (system locale), 700x475 px on screen at this PC's display scale. Screens captured: Ready, Working (stage bar 60%), Done, and "already installed" on a second run (buttons "Yeniden kur" and "MacDows'u aç").
- "Kur" (via UI Automation, no mouse movement): the install took about 2 s, exit code 0, uninstall key created with `DisplayVersion` 0.1.0 and `InstallLocation` `"C:\Users\fatal\AppData\Local\MacDows"` (quoted). Log `%LOCALAPPDATA%\MacDows-Setup\setup.log` had one line per stage (preparing 15, installing 60, finishing 95, done 100).
- "MacDows'u aç": `macdows.exe` started from the installed path, the installer window and process ended.
- Uninstall (`uninstall.exe /S`): exit 0, key and folder removed, `settings.json` kept, installer temp folder removed.

## Not verified (cannot be done here)
- A PC without WebView2 (the download path was exercised only with `MACDOWS_FORCE_NO_WEBVIEW2=1`, which downloads the 1.9 MB bootstrapper and deletes it; a real install of WebView2 was not run).
- No internet during the WebView2 step; upgrade over an older version; running the installer while MacDows is open through the installer window (NSIS itself was shown to end a running app in the first test).
- A very short console flash from a hidden child process cannot be ruled out by screenshots; all child processes are started with the no-window flag.
- SmartScreen behavior of the unsigned installer; the GitHub Actions release step (never run).
