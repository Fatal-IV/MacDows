# MacDows Core (sub-project 0) — Design

Date: 2026-11-02 · Status: awaiting user review

## Goal
A Windows 11 shell enhancement: a notch (top) and a dock (bottom) that grows,
step by step, into a full replacement of the taskbar, tray and Start menu.
Open source (MIT). Differentiator vs. existing tools: more features
(system monitor, clipboard history, launcher/Start replacement, notification center).
Written from scratch; no code is copied from other projects.

## Success criteria
- A user installs the signed installer, sees notch and dock, and uses them smoothly.
- The Windows taskbar can never be left permanently hidden.
- The app updates itself from GitHub Releases; every release has detailed notes.

## Decomposition
0 Core (this spec) · 1 System monitor · 2 Clipboard history · 3 Launcher / Start replacement · 4 Notification center.
Each gets its own spec, plan and release.

## Taskbar strategy (staged)
- v0.1: dock is an extra layer; the taskbar is untouched.
- v0.2: optional hide, with guard process and recovery shortcut.
- Later: work-area recalculation (SPI_SETWORKAREA), system tray, own Start UI opened by the Win key.
Windows' own Start menu cannot be modified; we provide our own launcher UI instead.

## Architecture
Single Tauri 2 app, multiple frameless transparent always-on-top windows
(`notch`, `dock`; later `settings`, `launcher`), hidden from the taskbar.
- UI (React + framer-motion): rendering and animation only.
- Bridge (Tauri commands/events): typed, one-way contracts.
- Rust modules, one responsibility each: `windows_list`, `fullscreen`, `monitors`,
  `taskbar`, `settings`, `updater`.
- `macdows-guard` (v0.2): tiny separate process whose only job is restoring the taskbar.

## Components
- Notch: collapsed pill; spring expand on hover (stiffness 380, damping 30).
  v0.1 content: clock, media panel (SMTC: artwork, title, play/pause/next/prev).
  Panels are pluggable components.
- Dock: running and pinned apps, click to focus, hover magnify, running indicator.
- Settings: visibility mode (Fixed/Smart/Peek). Default is Fixed: always visible, hidden only while an app is fullscreen (video, game). Smart (hide when overlapped) and Peek (reveal at screen edge) are opt-in. All modes hide for a fullscreen app, monitor selection, taskbar hiding (v0.2),
  language (tr/en), updates.

## Data flow
Event-driven, no polling. Rust emits `windows-changed`, `media-changed`, `fullscreen-changed`;
UI listens. UI sends commands (`focus_window`, `media_toggle`, `set_setting`).
Settings live in `%APPDATA%\MacDows\settings.json`.

## Error handling
- Taskbar: hide only when restore path is ready; restore on normal exit; guard restores on crash;
  on every start the state is checked and repaired.
- Errors are returned and logged to `%APPDATA%\MacDows\logs`, never silently swallowed.
- Missing media/icon/window info → component shows an empty state; app does not crash.
- If fullscreen detection fails, default to hidden.

## Testing
- Rust unit tests for pure logic (settings parsing, fullscreen decision, taskbar state machine).
- `tsc --noEmit` and component tests for the UI.
- Manual checklist on a real build before each release: taskbar hide/restore, multi-monitor,
  fullscreen game, crash simulation.
- CI on every tag: build and signature verification.

## Out of scope for v0.1
Taskbar hiding, tray, Start replacement, other sub-projects, multi-monitor beyond the primary.
