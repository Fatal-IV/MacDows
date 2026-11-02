# MacDows Installer and Logo — Design

Date: 2026-11-02 · Status: awaiting user review
Parent spec: `2026-11-02-core-design.md` (this spec adds the install experience and the brand mark; it does not change the app).

## Goal
Replace the plain NSIS wizard as the first-install download with a small branded installer that looks and behaves like the WinGuard installer (one borderless window, one "Install" button, progress, "Open"), and give MacDows its own logo. The logo also replaces the Tauri placeholder icons in the app (tray, window, installer, Start menu).

## Success criteria
- A user downloads `MacDows-Setup.exe`, clicks **Install** once, and MacDows is installed and can be opened from the last screen. No console window appears at any point.
- On a PC without WebView2 the installer installs it first (needs internet) and then shows its normal window.
- `MacDows-Setup.exe` is at most ~15 MB.
- Uninstall, Start menu entry and Apps list entry are the normal NSIS ones, with the new logo.
- Auto-update is unchanged: it still uses the plain NSIS package and `latest.json`.

## Decisions (user)
- Installer is a Tauri + React app (approach A), because it keeps the size small and the stack the same.
- Design is taken from `WinPhone GUARD` (`src/WinGuard.Setup`, the user's own code); the XAML is the visual reference, no code is reused.
- Logo is **variant 2**: violet gradient tile, black notch at the top with four equalizer bars, a white-to-ice-blue "M" in the middle, a black dock bar at the bottom with three dots.

## Logo
Source of truth: `branding/macdows-mark.svg` (96×96 viewBox), drawn from these parts:
- Tile: rounded rect 96×96, radius 22, linear gradient from `#2b1b5a` (top-left) to `#7a4dff` (bottom-right).
- Notch: path `M24 0 H72 V7 A13 13 0 0 1 59 20 H37 A13 13 0 0 1 24 7 Z`, fill `#05060a`. Four equalizer bars inside: x 38/44/50/56, widths 3, heights 7/13/9/5, colors `#7ee0ff`, `#fff`, `#7ee0ff`, `#fff`.
- "M": polyline `M26 58 V32 L48 48 L70 32 V58`, stroke 8, round caps and joins, stroke gradient `#ffffff` → `#7ee0ff` (top to bottom).
- Dock: rect x 14, y 68, 68×16, radius 8, fill `#05060a`; three dots r 3.6 at x 30/48/66, y 76, colors `#fff`, `#7ee0ff`, `#ff7a6b`.
- Small-size variants (24 px and 16 px) drop the equalizer bars and the dock dots and thicken the strokes (M stroke 11 and 13) so the mark stays readable. They are separate SVG files: `macdows-mark-24.svg`, `macdows-mark-16.svg`.

Generated outputs (script `scripts/make-icons.mjs`, committed with the SVGs so the build does not need design tools):
- `src-tauri/icons/*` regenerated for the app (`icon.ico` with 16/24/32/48/64/128/256, PNGs, `icon.icns`, Windows tile images) replacing the Tauri placeholders. The tray icon follows automatically.
- `installer/src-tauri/icons/*` for the installer.
- `installer/src/assets/mark.svg` for the in-window logo.

## Architecture
New folder `installer/` in the MacDows repo: its own Tauri 2 app (React + TypeScript front end, Rust back end), built separately from the main app. Output: `MacDows-Setup.exe`.

Units, each with one job:
1. **Launcher** (`installer/src-tauri/src/launcher.rs`, runs before any window exists, does not need WebView2). Checks whether WebView2 is installed; if not, downloads Microsoft's Evergreen bootstrapper and runs it silently, showing a plain native message "Gerekli bileşen kuruluyor… / Installing a required component…". Then starts the normal app.
   - Interface: `fn webview2_installed() -> bool`, `fn ensure_webview2() -> Result<(), SetupError>`.
2. **Installer core** (`installer/src-tauri/src/install.rs`). Extracts the embedded MacDows NSIS installer to a temp folder, runs it silently (`/S`, per-user), waits for the exit code and reports stages. Also answers "is MacDows already installed and which version" and launches the installed app.
   - Interface (Tauri commands): `install_state() -> InstallState`, `start_install()` (emits `install-stage` events), `launch_app()`.
3. **Texts** (`installer/src-tauri/src/texts.rs` mirrored in the UI): Turkish and English strings chosen from the Windows UI language; unknown languages use English.
4. **UI** (`installer/src/`): four screens (Ready, Working, Done, Error) in one borderless window. It only renders state and calls the three commands.

## Screens and look
Window 560×380, borderless, transparent outer margin 16, inner card radius 16, 1 px border `#22000000`, drop shadow (blur 24, offset 4, opacity .28), draggable by the card, ✕ at top right. Font Segoe UI Variable Text / Segoe UI; title in Georgia 40 px `#1B1C20`; muted text `#6D7078`, centered. Card background: gradient `#EEF1F6` → `#E9E6DF` (55%) → `#E4E0D8` at 0°→0.4,1.
- **Ready**: logo 76 px, "MacDows", slogan, accent button **Install / Kur** (bg `#0067C0`, hover `#005A9E`, radius 8, text white 14 px semibold), below it a 11 px note "By clicking Install you accept the license terms" (links to the MIT `LICENSE` on GitHub).
- **Working**: logo, "MacDows", progress bar 6 px (track `#1F000000`, fill `#0067C0`), status text 12 px. The bar is stage based and honest: Preparing 15% → Installing 60% (animated slowly while the installer runs) → Finishing 95% → 100%. It never shows a measured percentage because the NSIS silent mode reports none.
- **Done**: green circle `#2E9E6B` with ✓, title "Ready / Hazır", text "MacDows is installed.", accent button **Open MacDows**.
- **Error**: red circle `#C42B1C` with "!", title "Setup could not finish", one sentence for the cause, **Close** (quiet button: bg `#B3FFFFFF`, 1 px `#1F000000`) and **Try again**, and the log path in 10 px.
- If MacDows is already installed at the same or a newer version: Ready screen says "MacDows is already installed" with **Open MacDows** and **Reinstall**.
Slogan text: "Notch and dock for Windows" / "Windows için notch ve dock".

## Install behavior
- Per-user install (no administrator prompt). The embedded NSIS package is the one produced by `tauri build` with `bundle.windows.nsis.installMode = "currentUser"` (set explicitly).
- Silent run: `<setup>.exe /S`. Exit code 0 = success; any other code = error screen (message by code, default "The installer stopped with code N").
- "Installed" detection reads the per-user uninstall registry entry the NSIS package writes. The plan's first task verifies the exact key and value names against a real install before any code depends on them; the working assumption is `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\MacDows` with `DisplayVersion` and `InstallLocation`.
- If MacDows is running, the installer ends the `macdows.exe` process of the same user before running NSIS (the app keeps no unsaved state). If the process cannot be ended, the Error screen says MacDows must be closed first.
- "Open MacDows" starts `<InstallLocation>\macdows.exe`.
- Log file: `%LOCALAPPDATA%\MacDows-Setup\setup.log`, appended per run, one line per stage and error.

## WebView2 bootstrap
- Detection: registry value `pv` under `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` or the same key under `HKCU\Software\Microsoft\EdgeUpdate\Clients\...`; installed when the value exists and is not `0.0.0.0`. The plan verifies the key and the download address (`https://go.microsoft.com/fwlink/p/?LinkId=2124703`) on a real machine.
- Download with the Windows URL download API into `%TEMP%`, run `MicrosoftEdgeWebview2Setup.exe /silent /install`, then re-check detection.
- Needs internet on that PC. No offline mode in this version (the full runtime is ~170 MB).
- Failure shows a native message with the reason and the manual download link, then exits.

## Build and release
- `installer/` has its own `package.json` and `src-tauri`. Script `scripts/build-installer.mjs`: builds the main app (`bun run tauri build`), copies `src-tauri/target/release/bundle/nsis/MacDows_<version>_x64-setup.exe` into `installer/src-tauri/payload/`, then builds the installer with the version injected.
- `.github/workflows/release.yml` gains a step after the main build that runs the script and uploads `MacDows-Setup.exe` and `SHA256SUMS.txt` to the same GitHub release, next to the update package and `latest.json`. The release notes stay generated from `CHANGELOG.md`.
- The installer is not signed in this version (no code-signing certificate), so Windows SmartScreen may warn about an unknown publisher. This is documented in the README and the release notes.

## Errors
Each has one Turkish/English sentence and a log line: WebView2 could not be downloaded or installed; the embedded package could not be extracted (disk full or blocked); NSIS exited with a non-zero code; MacDows is running and could not be closed; the app could not be started after install. The installer never leaves a half-open window: every failure ends on the Error screen or the native message.

## Testing
- Unit tests (written first, `cargo test` in `installer/src-tauri`): language selection from a locale string; exit code → message mapping; "installed vs newer vs older" version comparison; WebView2 registry value interpretation (`0.0.0.0`, missing, valid); stage → progress percentage mapping; log line formatting.
- Type check: `tsc --noEmit` in `installer/`.
- Visual: screenshots of all four screens taken from the running installer, compared with the WinGuard XAML values above.
- Manual by the user (cannot be done in this environment): install on a PC without WebView2; install over a running MacDows; upgrade over an older version; uninstall from Apps; behavior on a machine with no internet.

## Out of scope
Code signing and SmartScreen reputation; offline WebView2; a custom uninstaller UI; installing for all users; in-app update UI (v0.2); installer language picker; per-monitor DPI polishing beyond Tauri defaults.

## Risks
- **Unsigned installer:** SmartScreen warning (cannot be fixed in code).
- **Registry key names** for NSIS and WebView2 are assumptions until verified on a real install (first plan task).
- **WebView2-less test** cannot be run here; the launcher logic is covered by unit tests only, the end-to-end path by the user's manual test.
- **Silent NSIS has no progress**, so the bar is staged by design.
