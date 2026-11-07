# Multi-monitor support: an island and a dock on every chosen monitor

Date: 2026-11-07. Status: design approved by the user in chat (2026-11-07); this document is the spec to review before the plan is written.

## Goal

Someone with more than one monitor can have the Dynamic Island and the dock on every monitor they choose, each one working on its own
monitor. Today both bars exist once and sit on the primary monitor.

Success looks like this on the user's PC (DISPLAY1 1920x1080 primary, DISPLAY6 2400x1350 at x=2400):

- In Settings > Appearance > Monitors, each monitor is a card with an **Island** and a **Dock** switch.
- With both switched on for the second monitor, it shows its own island at the top and its own dock at the bottom, and they behave like the
  primary ones (visibility modes, hover, panels, click-through).
- A fullscreen app on one monitor hides/reveals only that monitor's bars.
- Unplugging or plugging a monitor, or changing its resolution or scale, fixes itself within about two seconds.
- The Windows taskbar of a monitor that has a MacDows dock is hidden and comes back when MacDows exits (also after a crash).

## Decisions already made

| Question | Decision |
|---|---|
| What does "multi-monitor" mean | A separate island and dock on each chosen monitor (not one bar that moves, not one bar on a chosen monitor). |
| Defaults | Only the primary monitor has its island and dock on; every other monitor is off until the user switches it on. |
| Second Windows taskbar | Hidden on monitors that have a dock; included in this work, as the last phase. |
| Visibility mode, fullscreen choice, island type, scale | Stay global settings (one value for all monitors). Per-monitor values are out of scope. |
| Dock contents | Every dock shows the same pinned apps and the same running windows. Per-monitor window lists are out of scope. |
| Notification banners | Shown on every island. |

## Non-goals

- Different settings per monitor, a different pinned list per monitor, "show windows only where they are open".
- Moving a bar between monitors by dragging.
- Vertical or side-by-side monitor layouts beyond what `available_monitors()` gives (positions are used as they are).

## Current state (what the code assumes today)

- `tauri.conf.json` declares two windows, `notch` and `dock`. `place_notch` / `place_dock` (`lib.rs`) use `primary_monitor()`.
- Several modules look the window up by the fixed labels `"notch"` and `"dock"`: `hit_region.rs`, `visibility.rs`, `lib.rs` (setup,
  `island_focus`, tray panel), `preview.rs`, `app_catalog.rs`, `dock_menu.rs`, `hotkeys.rs` (`emit_to("dock", ...)`).
- The frontend picks its view from `getCurrentWindow().label` (`main.tsx`) and `useHitRegion("notch" | "dock", ...)` has the label fixed.
- `fullscreen.rs` already asks which monitor the foreground window is on, but keeps one global answer (`LAST`).
- `taskbar.rs` hides only `Shell_TrayWnd`; `Shell_SecondaryTrayWnd` windows are left alone.
- The preview, tray and app-catalog panels already use the dock's `current_monitor()`; they only need to be told which dock opened them.

## Design

### 1. Window model

- The existing `notch` and `dock` windows stay and belong to the **primary monitor**.
- Every other chosen monitor gets two windows built at run time with labels `notch-<n>` and `dock-<n>` (`n` counts up per run, the label is
  not persisted). They load the same `index.html`, with the same options as in `tauri.conf.json` (transparent, no decorations, always on
  top, no shadow, not focused, skip taskbar) and are registered with `windows_list::register_bar` like the first pair.
- A small helper module `bar.rs` is the one place that understands labels: `BarKind { Notch, Dock }`, `parse(label) -> Option<(BarKind,
  Option<u32>)>`, `is_bar(label)`, `label(kind, n)`. `hit_region::is_bar`, `visibility`, and the frontend call it instead of comparing
  strings. The frontend gets the same rule in `src/bar.ts` (`barKind(label)`).
- `main.tsx` chooses the view by `barKind(label)` for the two bar views; other labels behave as before.
- `useHitRegion` takes its label from the window (no literal `"notch"`/`"dock"`).

### 2. Monitors module (`monitors.rs`)

- `MonitorInfo { key, name, rect, work_area, scale, primary }` from `available_monitors()`.
- **Key:** the hardware device ID of the monitor (`EnumDisplayDevicesW` on the adapter name, `DeviceID` of the monitor child), because
  `\\.\DISPLAYn` can be renumbered by Windows. When it cannot be read, the `\\.\DISPLAYn` name is the key. The key is what Settings stores.
- **Desired state:** from `Settings.monitors` and the live monitors: `desired(monitors, cfg) -> Vec<(key, island: bool, dock: bool)>`.
  A monitor without a saved entry is `island: true, dock: true` when it is primary and `false`/`false` otherwise.
- **Reconcile plan (pure function):** `plan(desired, existing_windows) -> Vec<Action>` with `Create(kind, key)`, `Destroy(label)`,
  `Move(label, rect)`, `Show/Hide(label)`. Applying the plan is the only code with side effects. The static primary pair is never created
  or destroyed: when the primary monitor's island or dock is switched off, the window is hidden (the visibility loop treats "disabled" as
  "hidden" and never shows it).
- **When it runs:** at start, when the monitor settings change, and every 2 seconds it compares a signature of the monitor list
  (key, rect, scale, work area); a change runs the plan again. A bar whose monitor disappeared is destroyed; one whose rect changed is
  moved and resized.
- **Placement:** `place_notch`/`place_dock` become `place(kind, monitor)`: the island is centered on the top edge of the monitor, the dock
  on the bottom of the monitor's work area (the same formulas as now, on that monitor).
- **Mixed DPI:** a window is created at a position on the target monitor first, then sized, so it gets that monitor's scale factor; the
  frontend already measures in CSS pixels per window and uses its own `devicePixelRatio` (`hitRegion.ts`). This is the likeliest source
  of surprises with the 1920x1080 / 2400x1350 pair and is checked live.

### 3. Per-bar behaviour

- `visibility.rs` loops over every existing bar window, with its own `shown` state, its own overlap probe and reveal band computed from
  that window's rectangle (already how it works) and `enabled` from the monitor settings.
- `hit_region.rs` already keys everything by label; only `is_bar` and the two label lists (`["notch", "dock"]`) change to the set of
  live bar windows.
- **Fullscreen per monitor:** `fullscreen.rs` keeps `(monitor_rect, covers)` of the foreground window instead of one boolean and
  exposes `state_for(monitor_rect)`: `Some(covers)` for the monitor the foreground window is on, `Some(false)` for any other monitor,
  `None` while unknown. `visibility` and `hidden_in_fullscreen` ask `state_for` with their own monitor's rect. The
  `fullscreen-changed` event gains the monitor key so that frontends can ignore other monitors (to be checked in the plan: where the
  frontend uses it).
- The island and the dock components are independent React instances, one per window; data (media, notifications, status) reaches all of
  them through the app-wide events that already exist.

### 4. Panels, menus and shortcuts

- `preview`, `tray`, `addapp` and the dock menu keep one window each (they are never open on two monitors at once) and open on the monitor
  of the dock that opened them: the commands receive the calling `WebviewWindow` and use its `current_monitor()` instead of
  `get_webview_window("dock")`.
- `hotkeys.rs` sends `dock-shortcut` to the dock on the monitor under the pointer (the primary dock if there is none).
- `island_focus` focuses the island window that called it.
- The tray panel's "dock top" and the preview position use the calling dock's position (they already take `outer_position()` of "the
  dock").

### 5. Settings

- New field on `Settings`: `monitors: Vec<MonitorCfg>` with `MonitorCfg { key: String, name: String, island: bool, dock: bool }`,
  `#[serde(default)]`; an old file has an empty list and gets the defaults above. `name` is only the last seen display name, for showing
  a card for a monitor that is not plugged in at the moment (it stays in the list, is never deleted automatically).
- Command `get_monitors` returns the live monitors (key, name, size, position, primary, current island/dock flags); `set_setting` gets a
  `monitors` branch like the other settings. Changing it runs the reconcile plan at once.
- UI: a **Monitors** section on the Appearance page (below Layout). One card per monitor in the style of the island type tiles: a small
  drawing of the monitor with its position relative to the others, its name and resolution ("1 - 1920x1080, primary"), and two switches,
  *Island* and *Dock*. With one monitor the section is not shown. English and Turkish texts.

### 6. Windows taskbar on the other monitors (last phase)

- `taskbar.rs` hides `Shell_SecondaryTrayWnd` windows that are on a monitor with a dock, in addition to `Shell_TrayWnd`, and restores
  them on exit. `taskbar_plan.rs` (pure) records every hidden window handle in the state file; `restore_now` and the guard process
  restore all of them; on start, anything a previous run left hidden is restored first.
- A monitor whose dock is switched off keeps its Windows taskbar visible; plugging a monitor in while MacDows runs is handled by the same
  reconcile run (a new secondary taskbar is hidden only if that monitor has a dock).
- This is the riskiest part (a taskbar that stays hidden after a crash); it comes last, has its own unit tests on the plan functions, and
  is tested live with the guard (kill MacDows, the taskbars must come back).

## Testing

Unit tests (pure code, in the Rust modules and `node --test`):

- `bar.rs` label parsing and building (`notch`, `dock-2`, `settings`, `notch-` without a number).
- `monitors::desired` (defaults, saved entries, a saved monitor that is not plugged in) and `monitors::plan` (create, destroy, move,
  hide the primary pair, no change when nothing changed).
- Monitor key choice (device ID, fallback to the name).
- `fullscreen::state_for` (foreground monitor, other monitor, unknown).
- `taskbar_plan` with several hidden handles (state file round trip, restore list, stale state from another run).
- Settings: an old file without `monitors` loads with the defaults; a saved list round-trips.
- Frontend: `barKind`, and the Monitors section's logic (which cards, what is shown with one monitor) in a `*.test.ts`.

Live tests only the user can do, each phase: bars on the second monitor, mixed DPI, fullscreen on one monitor, unplugging and
replugging, the panels opening on the right monitor, hotkeys, the second taskbar and a crash recovery.

## Phases (each ends in a build the user can try with `npm run fast`)

1. **Windows and settings:** `bar.rs`/`bar.ts`, `monitors.rs` (keys, desired state, plan, apply, polling), per-bar visibility and hit
   region, the `monitors` setting, `get_monitors`, the Monitors section. Second monitor works with island and dock.
2. **Panels and shortcuts:** preview, tray, app catalog, dock menu, hotkeys and `island_focus` by calling window.
3. **Fullscreen per monitor.**
4. **Second Windows taskbar** (hide, restore, guard, crash recovery).

Each phase gets a changelog line (English and Turkish) when it ships. Phases 1 and 2 are needed for the feature to be usable; 3 and 4
can ship in a later release if the first two take long.

## Risks

- **Mixed DPI window creation:** a window built on a monitor with another scale can get the wrong size; mitigated by create, move,
  then size, and checked live on this PC's pair.
- **Taskbar left hidden after a crash:** mitigated by state-file recovery on start and by the guard process; last phase, own tests.
- **Event fan-out:** several islands each handle the same events (banners, media); they are independent React trees, so no shared
  state is lost, but the cost is duplicated work. Acceptable for two or three monitors.
- **Label lookups missed:** any `get_webview_window("dock")` left behind keeps pointing at the primary dock. A grep-based check in the
  plan lists them all (about ten today).
