# MacDows Taskbar Takeover — Design

Date: 2026-11-02 · Status: awaiting user review
Parent specs: `2026-11-02-core-design.md` (this is its "v0.2 taskbar" stage), `2026-11-02-installer-design.md`, `2026-11-02-update-check-design.md`.
Facts this design rests on: `docs/superpowers/notes/2026-11-02-taskbar-facts.md` (verified on a real PC).

## Goal
MacDows hides the Windows taskbar and takes its place: the dock is the app bar, the work area is given back to windows, and the things the tray offered stay reachable through buttons that open Windows' own flyouts. The taskbar always comes back, even when MacDows crashes or is removed.

## Success criteria
- With MacDows running, the taskbar is not visible, maximized windows use the full screen, and the dock/notch stay as they are.
- Quick settings, notifications with the calendar, and the hidden-icons popup open from buttons in the dock.
- Quitting MacDows (tray menu, update, uninstall), killing it, or a crash puts the taskbar back to **exactly** the state it was in before (auto-hide on or off).
- The setting is **on by default**; the user is told once, and can turn it off in Settings, from the tray menu, or with `macdows.exe --restore-taskbar`.
- A corrupt or missing state file never leaves the user without a taskbar.

## Decisions (user)
- Hiding is **on by default** (B), and the tray replacement is part of this work so a new user is not stranded.
- Tray: **T1**, native flyouts only. Redrawing other apps' tray icons in MacDows (T2) is a later job.

## Scope
In: hide + restore + guard + recovery, Explorer-restart handling, three shell buttons, first-run notice, Start with Windows. Out: T2 tray mirror, a MacDows Start menu / Win-key replacement (D), more than one monitor, the lock screen.

## How the taskbar is hidden
Both steps, in this order (see facts): set the appbar state to auto-hide (`SHAppBarMessage` `ABM_SETSTATE`, `ABS_AUTOHIDE`), then `ShowWindow(Shell_TrayWnd, SW_HIDE)`. Restore is the reverse: `SW_SHOW`, then the original appbar state.

## State file
`%APPDATA%\MacDows\taskbar-state.json`:
`{ "version": 1, "hidden_by_macdows": true, "was_autohide": false, "chevron_present": true, "app_pid": 1234 }`
- Written **before** anything is hidden; `was_autohide` is the real original value and is **never overwritten while `hidden_by_macdows` is true** (so a restart after a crash cannot record the already-modified state as the original).
- `chevron_present` is read once with UI Automation while the taskbar is still visible (the `Gizli Simgeleri Göster` item).
- Corrupt or unreadable file → treated as "no state".

## Units
1. **`src-tauri/src/taskbar_plan.rs` — pure logic (unit tested)**
   - `struct TaskbarState { version: u32, hidden_by_macdows: bool, was_autohide: bool, chevron_present: bool, app_pid: u32 }`, `fn state_from_json(&str) -> Option<TaskbarState>`, `fn state_to_json(&TaskbarState) -> String`.
   - `fn plan_hide(existing: Option<&TaskbarState>, current_autohide: bool, chevron_present: bool, pid: u32) -> HidePlan { state_to_save: TaskbarState, set_autohide: bool, hide_window: bool }` — keeps `was_autohide` from `existing` when it is `hidden_by_macdows`, otherwise takes `current_autohide`.
   - `fn plan_restore(state: Option<&TaskbarState>, window_hidden_now: bool) -> RestorePlan { show_window: bool, set_autohide: Option<bool>, clear_state: bool }` — state hidden by MacDows → show + set the stored original; no state → show only if the window is hidden, never guess an auto-hide value; state not hidden by MacDows → nothing.
   - `fn guard_should_restore(parent_alive: bool, state: Option<&TaskbarState>) -> bool`.
   - `fn hidden_icons_available(state: &TaskbarState, taskbar_hidden: bool) -> bool` (chevron was present).
   - `enum ShellAction { Notifications, QuickSettings, HiddenIcons }`, `fn key_steps(a: ShellAction) -> Vec<KeyStep>` where `KeyStep` is `Chord { win: bool, key: u8 }` or `Wait { ms: u32 }`: Notifications = Win+N, QuickSettings = Win+A, HiddenIcons = Win+B, wait 700 ms, Enter.
2. **`src-tauri/src/taskbar.rs` — effects:** Win32 calls (`FindWindow`, `SHAppBarMessage`, `ShowWindow`, `SendInput`), state file I/O under `%APPDATA%\MacDows`, `hide()`, `restore()`, `run_shell_action(ShellAction)` (command `shell_action`), a watcher for the registered message `TaskbarCreated` (Explorer restarted → hide again while the setting is on), and the exit hook that calls `restore()`.
3. **Guard:** a second small binary `macdows-guard.exe` shipped next to `macdows.exe` (in the NSIS package). The app starts it detached after hiding: `macdows-guard.exe --parent <pid>`. It waits for the parent process to end and then runs the same restore plan. NSIS ends `macdows.exe` by name on update/uninstall; the guard has a different name so it survives and restores. Fallback if the second binary cannot be bundled: `macdows.exe --guard <pid>` (plan verifies which works).
4. **Recovery:** `macdows.exe --restore-taskbar` (restore and exit, no window), a tray menu item "Restore Windows taskbar", the same check at every start (if the state says hidden but no MacDows is running, restore first), and an NSIS pre-uninstall hook that runs `macdows.exe --restore-taskbar`.
5. **Settings (`settings.rs`):** `replace_taskbar: bool` (default `true`), `taskbar_notice_shown: bool` (default `false`), `start_with_windows: bool` (default `true`, writes/removes `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` value `MacDows`).
6. **UI:** three buttons at the right end of the dock after a thin divider (bell = notifications, sliders = quick settings, chevron = hidden icons; the last only when `hidden_icons_available`), tooltips and labels in English/Turkish; Settings gets "Replace the Windows taskbar" and "Start with Windows" switches; the notch keeps the time and shows the battery percentage from `GetSystemPowerStatus`.
7. **First-run notice:** once, before the first hide, a native message: MacDows is replacing the taskbar; how to undo it (Settings switch, tray menu, `--restore-taskbar`). Shown again never (`taskbar_notice_shown`).

## Behavior details
- Turning the setting off restores immediately; turning it on hides immediately (and starts the guard).
- When a fullscreen app hides the dock (existing rule), the taskbar stays hidden; nothing flickers.
- Buttons send their keys to the foreground window as system shortcuts (`SendInput`); each action is a no-op with an error log if the foreground is elevated and refuses input.
- Hide only when the state file could be written. If hiding fails half way, run the restore plan before reporting.

## Errors
The state file cannot be written → do not hide, log, show nothing to the user. `Shell_TrayWnd` not found → nothing to hide, state not written. Guard cannot be started → still hide, log it (the app's own exit hook and the next-start check still restore). Explorer is not running → no taskbar to manage.

## Testing
- Unit tests first (`cargo test`): JSON round trip and corrupt/empty/unknown-version input; `plan_hide` keeps the original when already hidden by MacDows, records it otherwise, and for both a user with auto-hide on and off; `plan_restore` for all four cases (hidden by MacDows with each original, no state with the window hidden and visible, state not hidden by MacDows); `guard_should_restore`; `hidden_icons_available`; `key_steps` for the three actions (exact keys and the 700 ms wait).
- Real runs on this PC (each ends with the taskbar back): hide and restore from the setting; kill MacDows with Task Manager style termination and see the guard restore; kill both and see the next start restore; Explorer restart; the three buttons open the three flyouts; `--restore-taskbar`; install/update/uninstall paths; auto-hide originally on.
- Not verifiable here: a second monitor, a PC where the hidden-icons chevron is turned off, behavior after a reboot while hidden (checked as far as the guard and the next-start check cover it).

## Risks
- Hiding the taskbar is invasive; every failure path must end in "taskbar visible". The state file, the guard, the exit hook, the next-start check and the uninstall hook are five independent ways back.
- Win+B + Enter presses the first tray item if the chevron does not exist; the button is hidden in that case.
- The "start with Windows" default is a new decision for review: without it, after a reboot the taskbar would come back (auto-hide still on) until MacDows is started by hand.
- Windows updates can change the taskbar internals; the recovery commands use only documented window and appbar calls.
