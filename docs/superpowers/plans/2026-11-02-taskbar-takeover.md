# MacDows Taskbar Takeover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** MacDows hides the Windows taskbar by default, gives the work area back, offers three shell buttons (notifications, quick settings, hidden icons), and always restores the taskbar to its original state, even after a crash.

**Architecture:** All decisions live in pure functions and two small traits (`Backend` for Windows calls, `Store` for the state file), so hide and restore are unit tested with fakes. The real backend, a guard binary, the exit hook, the next-start check and the uninstall hook are five independent ways back to a visible taskbar.

**Tech Stack:** Rust (`windows` 0.61, `winreg` is not needed here; registry via `windows`), Tauri 2, React + TypeScript, NSIS hooks.

**Spec:** `docs/superpowers/specs/2026-11-02-taskbar-takeover-design.md` and the facts note `docs/superpowers/notes/2026-11-02-taskbar-facts.md` (read both).

## Global Constraints
- Hide order: auto-hide appbar state first, then `ShowWindow(Shell_TrayWnd, SW_HIDE)`. Restore order: `SW_SHOW`, then the stored original appbar state.
- State file `%APPDATA%\MacDows\taskbar-state.json`: `{ "version": 1, "hidden_by_macdows": bool, "was_autohide": bool, "chevron_present": bool, "app_pid": u32 }`, written **before** anything is hidden; `was_autohide` is never overwritten while `hidden_by_macdows` is true.
- Defaults: `replace_taskbar = true`, `taskbar_notice_shown = false`, `start_with_windows = true`; old `settings.json` files must still load.
- Shell buttons: Notifications = Win+N, QuickSettings = Win+A, HiddenIcons = Win+B, wait 700 ms, Enter. Never send Esc.
- Any failure path ends with the taskbar visible. Corrupt or missing state is never a reason to leave it hidden.
- Registry autostart value: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, name `MacDows`, value the quoted path of `macdows.exe`; only written in release builds.
- Commits authored only as `Fatal-IV`, no co-author or generated-by lines. Branch `feat/taskbar`.
- Checks before each commit: `cd src-tauri && cargo test`, `./node_modules/.bin/tsc --noEmit`, `node --test scripts/`.

## Review Focus
- Restart after a crash while the state file says hidden → the stored original is kept, never replaced by the modified auto-hide value (Task 1 test).
- User had auto-hide **on** originally → restore puts auto-hide back on and shows the window (Task 1 test).
- Corrupt, empty or unknown-version state file → taskbar ends up visible (Task 1 and Task 2 tests).
- State file cannot be written → nothing is hidden (Task 2 test with a failing `Store`).
- Hidden-icons button when the chevron was not present → not offered, because Enter would press the first tray icon (Task 1 test).

## File Structure
- `src-tauri/src/taskbar_plan.rs` pure logic and the `Backend`/`Store` traits with `hide`/`restore` orchestration; `src-tauri/src/taskbar.rs` the real backend, state store, Win32 calls, shell actions, watcher and commands; `src-tauri/src/bin/macdows-guard.rs` the guard; `src-tauri/installer-hooks.nsh`.
- `src/dock/Dock.tsx` and `Dock.css` (buttons), `src/settings/Settings.tsx`, `src/notch/Notch.tsx` (battery), `src/locales/{en,tr}.json`, `src/system.ts`.

---

### Task 1: Pure plans and key steps (tests first)
**Files:** Create `src-tauri/src/taskbar_plan.rs` (tests inside); Modify `src-tauri/src/settings.rs`, `src-tauri/src/lib.rs` (`mod taskbar_plan;`).
**Interfaces:** Produces:
- `struct TaskbarState { version: u32, hidden_by_macdows: bool, was_autohide: bool, chevron_present: bool, app_pid: u32 }` (`Serialize`, `Deserialize`, `Clone`, `PartialEq`, `Debug`); `fn state_from_json(&str) -> Option<TaskbarState>` (None for empty, corrupt or `version != 1`); `fn state_to_json(&TaskbarState) -> String`
- `struct HidePlan { state_to_save: TaskbarState, set_autohide: bool, hide_window: bool }`; `fn plan_hide(existing: Option<&TaskbarState>, current_autohide: bool, chevron_present: bool, pid: u32) -> HidePlan`
- `struct RestorePlan { show_window: bool, set_autohide: Option<bool>, clear_state: bool }`; `fn plan_restore(state: Option<&TaskbarState>, window_hidden_now: bool) -> RestorePlan`
- `fn guard_should_restore(parent_alive: bool, state: Option<&TaskbarState>) -> bool` (true only when the parent is gone and the state says hidden by MacDows)
- `fn hidden_icons_available(state: &TaskbarState, taskbar_hidden: bool) -> bool`
- `enum ShellAction { Notifications, QuickSettings, HiddenIcons }`, `enum KeyStep { Chord { win: bool, key: u8 }, Wait { ms: u32 } }`, `fn key_steps(a: ShellAction) -> Vec<KeyStep>`; `fn run_value(exe: &str) -> String` (the exe path wrapped in double quotes)
- `Settings.replace_taskbar`, `Settings.taskbar_notice_shown`, `Settings.start_with_windows`.
- [ ] **Step 1: Write the failing tests:** JSON round trip; `state_from_json("")`, `("{not json")`, a `"version": 2` file → `None`. `plan_hide(None, false, true, 7)` → saves `was_autohide=false`, `hidden_by_macdows=true`, `app_pid=7`, `set_autohide`, `hide_window`; `plan_hide(None, true, …)` → `was_autohide=true`; `plan_hide(Some(&hidden_state_with_was_autohide_false), true, …)` → saved `was_autohide` stays `false` (the review-focus case). `plan_restore`: hidden-by-MacDows with `was_autohide=false` → show + `set_autohide=Some(false)` + clear; with `was_autohide=true` → `Some(true)`; `None` with window hidden → show, `set_autohide=None`, no clear needed; `None` with window visible → nothing to do; a state with `hidden_by_macdows=false` → nothing. `guard_should_restore`: parent alive → false; parent gone + hidden state → true; parent gone + no state → false. `hidden_icons_available`: chevron present + hidden → true; chevron absent → false; taskbar not hidden → false. `key_steps`: `Notifications == [Chord{win:true,key:0x4E}]`, `QuickSettings == [Chord{win:true,key:0x41}]`, `HiddenIcons == [Chord{win:true,key:0x42}, Wait{ms:700}, Chord{win:false,key:0x0D}]`; `run_value(r"C:\A B\macdows.exe") == "\"C:\\A B\\macdows.exe\""`. Settings: defaults `true/false/true`; a v0.1.0 settings file still loads.
- [ ] **Step 2:** `cd src-tauri && cargo test taskbar_plan settings` → Expected: FAIL (items missing).
- [ ] **Step 3:** Implement. Commit `feat: taskbar plans and settings fields`.

### Task 2: Hide and restore over traits (tests first)
**Files:** Modify `src-tauri/src/taskbar_plan.rs`.
**Interfaces:** Consumes Task 1. Produces `trait Backend { fn autohide(&self) -> bool; fn set_autohide(&self, on: bool); fn hide_window(&self); fn show_window(&self); fn window_hidden(&self) -> bool; fn chevron_present(&self) -> bool; }`, `trait Store { fn load(&self) -> Option<TaskbarState>; fn save(&self, s: &TaskbarState) -> std::io::Result<()>; fn clear(&self); }`, `fn hide(b: &impl Backend, s: &impl Store, pid: u32) -> Result<(), HideError>` (`HideError::StateNotSaved`), `fn restore(b: &impl Backend, s: &impl Store)`.
- [ ] **Step 1: Write the failing tests with in-memory fakes that record calls:** `hide` saves the state **before** any `set_autohide`/`hide_window` call (assert call order); with a failing `Store.save` → returns `Err(StateNotSaved)` and no backend call happened; `hide` then `restore` with original auto-hide off ends with `show_window` called, autohide false, state cleared; the same with original auto-hide on ends with autohide true; hide twice in a row (crash-restart case) keeps the first `was_autohide`; `restore` with a corrupt/absent state and a hidden window shows it and does not touch auto-hide.
- [ ] **Step 2:** `cargo test taskbar_plan` → FAIL. **Step 3:** implement the traits and functions on top of `plan_hide`/`plan_restore`. **Step 4:** PASS. Commit `feat: hide and restore orchestration`.

### Task 3: Real Windows backend, state file and shell actions
**Files:** Create `src-tauri/src/taskbar.rs`; Modify `src-tauri/src/lib.rs`, `src-tauri/Cargo.toml` (windows features `Win32_UI_Input_KeyboardAndMouse`, `Win32_UI_Shell`, `Win32_System_Power`, `Win32_UI_Accessibility` if needed).
**Interfaces:** Consumes Tasks 1–2. Produces `struct WinBackend` (`impl Backend`: `FindWindow("Shell_TrayWnd")`, `SHAppBarMessage` `ABM_GETSTATE`/`ABM_SETSTATE`, `ShowWindow`, `IsWindowVisible`, chevron lookup through UI Automation `AutomationId` `SystemTrayIcon` named like "Gizli Simgeleri Göster"/"Show hidden icons" while visible), `struct FileStore` (`impl Store` at `%APPDATA%\MacDows\taskbar-state.json`, atomic write through a temp file + rename), commands `shell_action(action: String)`, `get_system_status() -> SystemStatus { battery_percent: Option<u8>, hidden_icons_available: bool, taskbar_hidden: bool }`, and `pub fn restore_now()` for the CLI and the guard.
- [ ] **Step 1: Write the failing test** `file_store_roundtrip_and_corrupt_file`: with a non-ASCII temp dir, `save` then `load` returns the same state; a file with garbage loads as `None`; `clear` removes the file; a second `save` replaces the first atomically (no leftover temp file).
- [ ] **Step 2:** `cargo test taskbar` → FAIL. **Step 3:** implement `FileStore`, `WinBackend`, key sending with `SendInput` following `key_steps` (map `Wait` to `thread::sleep`), the `TaskbarCreated` registered-message watcher thread that calls `hide` again while `replace_taskbar` is on, battery via `GetSystemPowerStatus` (255 → `None`). **Step 4:** `cargo test` → PASS; `cargo check` clean. Commit `feat: windows taskbar backend`.

### Task 4: Guard binary, CLI flags and packaging
**Files:** Create `src-tauri/src/bin/macdows-guard.rs`; Modify `src-tauri/Cargo.toml` (`default-run = "macdows"`, the `[[bin]]` entries), `src-tauri/src/main.rs`, `src-tauri/tauri.conf.json` (`bundle.resources`).
**Interfaces:** Consumes `taskbar::restore_now`, `guard_should_restore`. Produces `macdows-guard.exe --parent <pid>` (waits for the process to end, then restores if `guard_should_restore`, then exits), `macdows.exe --restore-taskbar` (restore and exit with no window), and `macdows.exe --guard <pid>` as the fallback.
- [ ] **Step 1: Write the failing test** `parse_args_recognizes_the_recovery_flags` in `src-tauri/src/cli.rs`: `parse(&["--restore-taskbar"]) == Cli::RestoreTaskbar`; `parse(&["--guard","123"]) == Cli::Guard(123)`; `parse(&["--guard","x"])` and `parse(&[])` → `Cli::Normal`.
- [ ] **Step 2:** `cargo test cli` → FAIL. **Step 3:** implement `cli.rs`, the guard binary and the flag handling in `main.rs` (before the Tauri builder).
- [ ] **Step 4 (verify, record in `docs/superpowers/notes/2026-11-02-taskbar-facts.md`):** `bun run tauri build` produces `macdows-guard.exe` inside the NSIS package next to `macdows.exe` (install it, with the user's permission, then uninstall); dev builds fall back to `macdows.exe --guard`. If bundling the second binary does not work, switch to the fallback and record why. Commit `feat: guard process and recovery flags`.

### Task 5: Wiring and settings
**Files:** Modify `src-tauri/src/lib.rs`, `src-tauri/src/settings.rs` usage, `src-tauri/src/taskbar.rs`.
**Interfaces:** Consumes Tasks 1–4. Produces: at `setup`, the next-start check (state says hidden but no live `app_pid` → `restore` first), then `hide` + guard spawn when `replace_taskbar`; the first-run native message (once, `taskbar_notice_shown`) before the first hide; `set_setting("replace_taskbar", …)` hides/restores immediately; the autostart registry value for `start_with_windows` (release builds only, uses `run_value`); a tray menu item "Restore Windows taskbar" (English/Turkish via `i18n.rs`, with a test for both labels); `restore` on normal exit.
- [ ] **Step 1: Write the failing test** `tray_labels_include_the_restore_item` in `i18n.rs` for `tr` and `en` (unknown → English). **Step 2:** FAIL. **Step 3:** implement the wiring. **Step 4:** `cargo test` → PASS. Commit `feat: wire the taskbar takeover`.

### Task 6: UI and uninstall hook
**Files:** Create `src/system.ts`, `src-tauri/installer-hooks.nsh`; Modify `src/dock/Dock.tsx`, `src/dock/Dock.css`, `src/settings/Settings.tsx`, `src/notch/Notch.tsx`, `src/locales/en.json`, `src/locales/tr.json`, `src-tauri/tauri.conf.json` (`bundle.windows.nsis.installerHooks`).
**Interfaces:** Consumes `shell_action`, `get_system_status`. `src/system.ts` exports `shellAction(a: "notifications" | "quick_settings" | "hidden_icons")`, `useSystemStatus()`.
- [ ] **Step 1:** Dock: divider then three 40 px buttons (bell, sliders, chevron icons as inline SVG) with tooltips; the chevron only when `hidden_icons_available`; Notch shows the battery percentage next to the time when known; Settings: "Replace the Windows taskbar" and "Start with Windows" switches with English/Turkish strings.
- [ ] **Step 2:** `installer-hooks.nsh` with `!macro NSIS_HOOK_PREUNINSTALL` running `ExecWait '"$INSTDIR\macdows.exe" --restore-taskbar'`.
- [ ] **Step 3:** `./node_modules/.bin/tsc --noEmit` → clean. Commit `feat: system buttons, settings and uninstall hook`.

### Task 7: Real runs on this PC
**Files:** evidence appended to `docs/superpowers/notes/2026-11-02-taskbar-facts.md`.
- [ ] **Step 1:** Ask the user's permission (the taskbar will vanish; every step ends with it back; settings are backed up and restored). Run MacDows with the setting on: first-run message once, taskbar hidden, dock and maximized windows use the full screen (screenshots), the three buttons open Win+N, Win+A and the hidden-icons popup (screenshots), turning the setting off brings the taskbar back at once.
- [ ] **Step 2:** Kill `macdows.exe` forcibly → the guard restores the taskbar within about a second. Kill both → start MacDows again → the next-start check restores (then hides again). `macdows.exe --restore-taskbar` works with MacDows not running. Start with auto-hide originally **on** (set it, run, quit) → it is on again afterwards.
- [ ] **Step 3:** Restart Explorer (`taskkill /f /im explorer.exe`, then start it) while hidden → the taskbar is hidden again. Install the NSIS package and uninstall while hidden → the taskbar comes back (uninstall hook). Record results and everything not verified (second monitor, chevron turned off, reboot). Commit `docs: taskbar verification notes`.

### Task 8: Docs
**Files:** Modify `CHANGELOG.md`, `README.md`.
- [ ] **Step 1:** `[Unreleased]` entry (Added): taskbar replacement on by default, three shell buttons, battery on the notch, Start with Windows, the five ways back, recovery command and tray item, how to switch it off. README: a short "Windows taskbar" section with the recovery command. Run `node scripts/extract-notes.mjs Unreleased` and confirm the entries. Commit `docs: taskbar takeover notes`.

---
## Self-review
- **Spec coverage:** hiding and state → Tasks 1–3; guard, recovery, packaging → Task 4; settings, first-run notice, autostart, tray item, exit hook, next-start check → Task 5; dock buttons, battery, switches, uninstall hook → Task 6; success criteria and real runs → Task 7; docs → Task 8. Out of scope respected (T2, Start menu, second monitor).
- **Type consistency:** `TaskbarState`, `HidePlan`, `RestorePlan`, `Backend`, `Store`, `ShellAction`, `KeyStep`, `run_value`, `shell_action`, `get_system_status` keep the same names and fields across Tasks 1–6.
- **Not verifiable here:** a second monitor, a chevron turned off, a reboot while hidden; stated in Task 7.
