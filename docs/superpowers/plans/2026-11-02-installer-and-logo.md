# MacDows Installer and Logo Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `MacDows-Setup.exe`, a small branded Tauri installer (Ready → Working → Done/Error) that installs the existing NSIS package silently and bootstraps WebView2, plus the MacDows logo used by the app, tray and installer.

**Architecture:** `installer/` is a separate Tauri 2 app. Pure decisions live in one Rust module (`logic.rs`) covered by unit tests; Windows effects live in `launcher.rs` (WebView2) and `install.rs` (extract, run NSIS, detect, launch); the React UI only renders state and calls three commands. The logo is one SVG source rendered to icons by a script.

**Tech Stack:** Tauri 2, Rust (`windows` 0.61, `winreg`), React 19 + TypeScript + Vite, Bun, `@resvg/resvg-js`, `png-to-ico`.

**Spec:** `docs/superpowers/specs/2026-11-02-installer-design.md` (read it with this plan; parent spec `2026-11-02-core-design.md`).

## Global Constraints
- Windows 11, per-user install, no administrator prompt; NSIS `bundle.windows.nsis.installMode = "currentUser"`.
- `MacDows-Setup.exe` at most ~15 MB; no console window at any point (`windows_subsystem = "windows"`, hidden child processes).
- Window 560×380, borderless, card radius 16, border `#22000000`, shadow blur 24 / offset 4 / opacity .28; card gradient `#EEF1F6` → `#E9E6DF` (55%) → `#E4E0D8`; accent `#0067C0` (hover `#005A9E`); success `#2E9E6B`; error `#C42B1C`; muted `#6D7078`; title Georgia 40 px `#1B1C20`; quiet button bg `#B3FFFFFF`, border `#1F000000`.
- Progress is staged and never shows a measured percentage: Preparing 15, Installing 60, Finishing 95, Done 100.
- Languages: Turkish when the Windows UI language starts with `tr`, otherwise English.
- Logo: variant 2 exactly as specified in the spec; 24 px and 16 px use separate simplified SVGs.
- Auto-update keeps using the plain NSIS package; the installer is an extra release asset.
- Commits authored only as `Fatal-IV`, no co-author or generated-by lines. License MIT. The WinGuard installer is a visual reference only; no code is copied.
- Checks before each commit: `cd installer/src-tauri && cargo test`, `cd installer && ./node_modules/.bin/tsc --noEmit`.

## Review Focus
- Embedded payload missing at build time → the build script fails with a clear message (Task 8 test).
- NSIS exits with a non-zero code → Error screen with a mapped sentence, never a silent close (Task 3 test).
- Bundled version older than the installed one → no downgrade, Ready screen offers Open (Task 3 test).
- Windows language neither Turkish nor English (for example `de-DE`) or empty → English (Task 3 test).
- User profile or temp path containing non-ASCII characters (for example `C:\Users\Çağrı Şen\AppData\Local\Temp`) → payload extraction and launch still work (Task 5 test).

## File Structure
- `branding/macdows-mark.svg`, `macdows-mark-24.svg`, `macdows-mark-16.svg` — logo sources.
- `scripts/make-icons.mjs` — renders SVGs to icon files.
- `scripts/build-installer.mjs` — builds app, embeds payload, builds installer.
- `installer/` — package.json, vite config, `src/` (React), `src-tauri/` (Rust: `logic.rs`, `launcher.rs`, `install.rs`, `lib.rs`, `main.rs`, `payload/`).

---

### Task 1: Verify the real Windows facts before coding against them
**Files:** Create `docs/superpowers/notes/2026-11-02-installer-facts.md`
**Interfaces:** Produces the confirmed registry key and value names that Tasks 3 and 5 use. No code.
- [ ] **Step 1:** Read-only on this PC: list `HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}` and the `HKCU` equivalent, record the `pv` value. Confirm `https://go.microsoft.com/fwlink/p/?LinkId=2124703` redirects to an `.exe` (`curl -sIL`, record final URL and size). Expected: `pv` present; final URL ends in `MicrosoftEdgeWebview2Setup.exe`.
- [ ] **Step 2:** Ask the user for permission, then run the existing `src-tauri/target/release/bundle/nsis/MacDows_0.1.0_x64-setup.exe /S`. Record the exit code, the new `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\*` key whose `DisplayName` is MacDows (name, `DisplayVersion`, `InstallLocation`, `UninstallString`), the installed exe path, and whether a running `macdows.exe` blocks the run. Then run the recorded uninstall command silently and confirm the key is gone.
- [ ] **Step 3:** Write the findings (exact key names, values, URL) to the notes file. Expected: the file contains every key and value later tasks cite. Commit `docs: record installer registry facts`.

### Task 2: Logo sources and icon generation
**Files:** Create `branding/macdows-mark.svg`, `branding/macdows-mark-24.svg`, `branding/macdows-mark-16.svg`, `scripts/make-icons.mjs`, `scripts/make-icons.test.mjs`; Modify `package.json` (dev deps and `"icons": "node scripts/make-icons.mjs"`); regenerates `src-tauri/icons/*`.
**Interfaces:** Produces `installer/src/assets/mark.svg` (copy of the main mark) and icon sets for app and installer; `make-icons.mjs` exports nothing, it is a CLI.
- [ ] **Step 1: Write the failing test** `scripts/make-icons.test.mjs` (node:test): after running the script, `src-tauri/icons/icon.ico` exists and its directory lists the sizes `[16,24,32,48,64,128,256]`; `src-tauri/icons/32x32.png` is 32×32; `branding/macdows-mark.svg` contains the strings `#2b1b5a`, `#7a4dff`, `#05060a`, `#7ee0ff`, `#ff7a6b`.
- [ ] **Step 2:** Run `node --test scripts/make-icons.test.mjs`. Expected: FAIL (files missing).
- [ ] **Step 3:** Write the three SVGs from the spec geometry. Implement the script: render 16/24 from the small SVGs and 32 to 1024 from the main SVG with `@resvg/resvg-js`, assemble `icon.ico` with `png-to-ico`, run `bun run tauri icon` on the 1024 PNG for the remaining Tauri formats, then overwrite `icon.ico` with the size-specific one; copy outputs to `installer/src-tauri/icons/` and `installer/src/assets/mark.svg`.
- [ ] **Step 4:** Run the test. Expected: PASS. Screenshot the three SVGs with headless Edge and read the image to confirm they match variant 2 (M gradient, notch bars, dock dots, simplified small sizes).
- [ ] **Step 5:** Run `bun run tauri dev` briefly: the tray icon shows the new mark. Commit `feat: MacDows logo and generated icons`.

### Task 3: Pure installer logic (Rust, tests first)
**Files:** Create `installer/src-tauri/src/logic.rs` (tests inside).
**Interfaces:** Produces:
- `enum Lang { Tr, En }`, `fn pick_lang(locale: &str) -> Lang`
- `enum Decision { Fresh, Reinstall, Older, Newer }` and `fn decide(installed: Option<&str>, bundled: &str) -> Decision` (`Reinstall` = same version, `Older` = installed older than bundled, `Newer` = installed newer)
- `fn exit_code_message(lang: Lang, code: i32) -> String`
- `fn webview2_present(pv: Option<&str>) -> bool`
- `enum Stage { Preparing, Installing, Finishing, Done }`, `fn stage_percent(s: Stage) -> u8`
- `fn log_line(timestamp: &str, level: &str, message: &str) -> String` returning `[<timestamp>] <LEVEL uppercase> <message>`
- [ ] **Step 1: Write the failing tests:** `pick_lang("tr-TR")==Tr`, `pick_lang("TR")==Tr`, `pick_lang("de-DE")==En`, `pick_lang("")==En`; `decide(None,"0.1.0")==Fresh`, `decide(Some("0.1.0"),"0.1.0")==Reinstall`, `decide(Some("0.0.9"),"0.1.0")==Older`, `decide(Some("0.2.0"),"0.1.0")==Newer`, `decide(Some("garbage"),"0.1.0")==Older`; `exit_code_message(En,2)` contains `"2"`, and `exit_code_message(Tr,1)` is Turkish (contains `"kurulum"`), code 0 is never mapped (debug_assert); `webview2_present(None)==false`, `Some("")==false`, `Some("0.0.0.0")==false`, `Some("131.0.2903.51")==true`; `stage_percent` returns 15/60/95/100; `log_line("12:00:00","error","boom")=="[12:00:00] ERROR boom"`.
- [ ] **Step 2:** `cd installer/src-tauri && cargo test logic` → Expected: FAIL (module missing).
- [ ] **Step 3:** Implement. Version comparison parses `major.minor.patch`, ignores extra parts, unparseable installed version counts as `Older`.
- [ ] **Step 4:** `cargo test` → PASS. Commit `feat(installer): pure setup logic`.

### Task 4: Installer app scaffold
**Files:** Create `installer/package.json`, `installer/vite.config.ts`, `installer/tsconfig.json`, `installer/index.html`, `installer/src-tauri/{Cargo.toml,tauri.conf.json,build.rs,capabilities/default.json,src/main.rs,src/lib.rs}`.
**Interfaces:** Consumes `logic.rs` (Task 3). Produces a runnable window labeled `setup` (560×380, `decorations:false`, `transparent:true`, `resizable:false`, center) and the three command names registered with empty bodies returning `Err("not implemented")`: `install_state`, `start_install`, `launch_app`.
- [ ] **Step 1:** Create the files; `tauri.conf.json` product name `MacDows Setup`, identifier `dev.macdows.setup`, bundle icons from Task 2, `windows_subsystem = "windows"` in `main.rs`, release profile `opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"`.
- [ ] **Step 2:** `cd installer && bun install && cd src-tauri && cargo check`. Expected: no errors.
- [ ] **Step 3:** `bun run tauri dev` in `installer/`; confirm an empty borderless window opens. Commit `feat(installer): scaffold`.

### Task 5: Install core (Windows effects)
**Files:** Create `installer/src-tauri/src/install.rs`; Modify `lib.rs`; Create `installer/src-tauri/payload/.gitkeep` (payload exe is git-ignored).
**Interfaces:** Consumes `Decision`, `Stage`, `stage_percent`, `exit_code_message`, `log_line`, `pick_lang`. Produces:
- `fn extract_payload(dest_dir: &Path) -> Result<PathBuf, SetupError>` (writes the embedded `include_bytes!` NSIS exe)
- `fn run_nsis(exe: &Path) -> Result<i32, SetupError>` (hidden window, `/S`, waits)
- `fn read_installed() -> Option<InstalledInfo { version: String, location: PathBuf }>` using the key recorded in Task 1
- `fn end_running_app() -> Result<(), SetupError>`
- Tauri commands: `install_state() -> InstallInfo { decision: String, installed_version: Option<String>, bundled_version: String }`, `start_install()` emitting `install-stage { stage: String, percent: u8 }` then `install-done` or `install-error { message: String }`, `launch_app()`.
- [ ] **Step 1: Write the failing test** `extract_payload_works_with_non_ascii_dir`: create a temp directory named `Çağrı Şen ı` under `std::env::temp_dir()`, call `extract_payload`, assert the returned file exists, its length equals the embedded bytes length, and its name ends with `.exe`. A second test `log_file_is_appended`: two `write_log` calls create two lines in one file.
- [ ] **Step 2:** `cargo test install` → FAIL.
- [ ] **Step 3:** Implement the functions and commands. Approach: the embedded exe path comes from the build script's `payload/` folder via `include_bytes!` behind `cfg(not(test))` plus a tiny stub constant for tests; log file at `%LOCALAPPDATA%\MacDows-Setup\setup.log`; the 100 ms stage animation for Installing runs on a separate thread and stops when NSIS returns.
- [ ] **Step 4:** `cargo test` → PASS. Commit `feat(installer): install core`.

### Task 6: WebView2 launcher
**Files:** Create `installer/src-tauri/src/launcher.rs`; Modify `main.rs` (call before building the Tauri app).
**Interfaces:** Consumes `webview2_present`. Produces `fn webview2_installed() -> bool` (reads `pv` from HKLM/WOW6432Node then HKCU), `fn ensure_webview2() -> Result<(), SetupError>` (download with the Windows URL download API to `%TEMP%`, run `MicrosoftEdgeWebview2Setup.exe /silent /install`, re-check), and a native message box on failure with the reason and the manual download link.
- [ ] **Step 1: Write the failing test** `pv_from_registry_values_prefers_machine_then_user`: a pure helper `fn first_pv(machine: Option<String>, user: Option<String>) -> Option<String>` returns machine when present, else user.
- [ ] **Step 2:** `cargo test launcher` → FAIL. **Step 3:** implement helper and the effectful functions. **Step 4:** `cargo test` → PASS.
- [ ] **Step 5:** Manual check on this PC: `webview2_installed()` is true (logged at start). The download path is exercised by calling `ensure_webview2` against a fake missing key through an env override `MACDOWS_FORCE_NO_WEBVIEW2=1` that skips the real install and only downloads to temp, then deletes the file. Commit `feat(installer): WebView2 bootstrap`.

### Task 7: Installer UI
**Files:** Create `installer/src/{main.tsx,App.tsx,App.css,api.ts,i18n.ts}`, `installer/src/assets/mark.svg` (from Task 2).
**Interfaces:** Consumes the three commands and the events from Task 5. `api.ts` exports `installState()`, `startInstall()`, `launchApp()`, `onStage(cb)`, `onDone(cb)`, `onError(cb)`, with a mock used when not running inside Tauri (query `?screen=ready|working|done|error|installed` selects a screen for screenshots).
- [ ] **Step 1:** Build the four screens exactly per the spec values in Global Constraints: layout, fonts, buttons, ✕ at top right, draggable card (`data-tauri-drag-region`), 560×380.
- [ ] **Step 2:** Strings in `i18n.ts` for Turkish and English (Ready/Install, Working statuses, Done text, Error title and buttons, "already installed" variant, slogan "Windows için notch ve dock" / "Notch and dock for Windows").
- [ ] **Step 3:** `./node_modules/.bin/tsc --noEmit`. Expected: clean.
- [ ] **Step 4:** Run Vite, open each `?screen=` in headless Edge at 592×412 (window plus margin), save five PNGs, read them and compare against the XAML values from `WinPhone GUARD/src/WinGuard.Setup/SetupWindow.xaml`. Fix visible differences. Commit `feat(installer): UI`.

### Task 8: Build script, release workflow, docs
**Files:** Create `scripts/build-installer.mjs`, `scripts/build-installer.test.mjs`; Modify `.github/workflows/release.yml`, `src-tauri/tauri.conf.json` (`nsis.installMode`), `.gitignore`, `README.md`, `CHANGELOG.md` (`[Unreleased]`).
**Interfaces:** Produces `installer/src-tauri/payload/` filled with the NSIS exe, `installer/dist/MacDows-Setup.exe`, `SHA256SUMS.txt`.
- [ ] **Step 1: Write the failing test** (node:test): `findPayload(bundleDir, version)` throws a message containing `"MacDows_0.1.0_x64-setup.exe"` and `"not found"` when the exe is missing, and returns the path when present (temp dir fixture).
- [ ] **Step 2:** Run it → FAIL. **Step 3:** Implement `findPayload` and the script: build the app, copy the payload, inject the version into the installer build, build the installer, write the SHA-256 file.
- [ ] **Step 4:** Set `bundle.windows.nsis.installMode` to `"currentUser"`. Add a release workflow step that runs the script and uploads `MacDows-Setup.exe` and `SHA256SUMS.txt` with `gh release upload`. README: install section, SmartScreen note, WebView2 note. CHANGELOG `[Unreleased]`: logo, branded installer.
- [ ] **Step 5:** Run the test → PASS. Commit `feat: build and release the installer`.

### Task 9: End-to-end verification on this PC
**Files:** none (evidence goes to `docs/superpowers/notes/2026-11-02-installer-facts.md`).
- [ ] **Step 1:** Run `node scripts/build-installer.mjs`. Expected: `installer/dist/MacDows-Setup.exe` exists and is at most 15 MB; report the exact size.
- [ ] **Step 2:** With the user's permission, run the installer: click Install through the real window (screenshots of Ready, Working, Done), confirm MacDows starts from "Open MacDows", the Apps list shows the logo, and no console window appeared. Run it again: the "already installed" screen appears.
- [ ] **Step 3:** Uninstall from the recorded command and confirm cleanup. Record every result, including anything not verified (clean PC without WebView2, no internet, upgrade over an older version), in the notes file and in the final report. Commit `docs: installer verification notes`.

---
## Self-review
- **Spec coverage:** success criteria → Tasks 8–9; logo → Task 2; architecture units → Tasks 3, 5, 6, 7; screens → Task 7; install behavior and registry → Tasks 1, 5; WebView2 → Tasks 1, 6; build and release → Task 8; errors → Tasks 3, 5, 7; testing → every task; out-of-scope respected.
- **Type consistency:** `Decision`, `Stage`, `stage_percent`, `exit_code_message`, `log_line`, `pick_lang`, `webview2_present` are defined in Task 3 and used with the same names in Tasks 5–7; command names `install_state`, `start_install`, `launch_app` and event names `install-stage`, `install-done`, `install-error` are identical in Tasks 4, 5 and 7.
- **Not verifiable here:** a PC without WebView2, no internet, upgrade over an older version, SmartScreen behavior. These stay manual for the user and are reported as unverified.
