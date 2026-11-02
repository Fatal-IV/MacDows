# MacDows

A notch and a dock for Windows, built with Tauri 2, Rust and React.

## Install

1. Download `MacDows-Setup.exe` from the [latest release](https://github.com/Fatal-IV/MacDows/releases/latest).
2. Run it and press **Install**. No administrator permission is needed; MacDows is installed for your user.
3. Press **Open MacDows**. The tray icon (in the hidden icons) opens Settings.

Verify the download: each release has a `SHA256SUMS.txt`. Compare with

```powershell
Get-FileHash .\MacDows-Setup.exe -Algorithm SHA256
```

Notes:
- The installer is not code-signed yet, so Windows SmartScreen may show an "unknown publisher" warning. Choose **More info → Run anyway** if the checksum matches.
- MacDows needs the WebView2 runtime. Windows 11 has it. If it is missing, the installer downloads and installs it first (internet required).
- Uninstall from Windows Settings → Apps.

## Updates

MacDows checks for a new release a little after it starts and then every 12 hours. When there is one, a small blue dot appears on the notch and **Settings → Updates** shows what is new. Nothing is installed until you press **Update and restart**. Updates are verified with the MacDows signing key. You can turn the automatic check off in the same card.

## Develop

```
bun install
bun run tauri dev
```

Build the installer: `node scripts/build-installer.mjs` (needs `TAURI_SIGNING_PRIVATE_KEY`). Regenerate the icons from `branding/*.svg`: `bun run icons`.

License: MIT
