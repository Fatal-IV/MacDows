# Multi-monitor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An island and a dock on every monitor the user chooses, each working on its own monitor.

**Architecture:** The existing `notch` and `dock` windows stay and belong to the primary monitor; every other chosen monitor gets `notch-<n>` / `dock-<n>` windows built at run time. One module (`bar.rs`) understands window labels, one (`monitors.rs`) knows the monitors and turns "what the settings want" into create/destroy actions with a pure planner. Visibility, hit region, fullscreen and panels work per bar window.

**Tech Stack:** Rust (Tauri 2, `windows` crate 0.61, `serde`), React + TypeScript, `node --test` for frontend logic, `cargo test --lib` for Rust.

**Spec:** `docs/superpowers/specs/2026-11-07-multi-monitor-design.md`

## Global Constraints

- Defaults: only the primary monitor has island and dock on; every other monitor is off until switched on (spec, Decisions).
- Visibility mode, fullscreen choice, island type and UI scale stay global; dock contents are identical on every dock; banners show on every island.
- The static windows `notch` and `dock` are never created or destroyed at run time; when the primary monitor's bar is disabled they are hidden, not closed.
- Monitor key = hardware device ID of the monitor, lowercased; the `\\.\DISPLAYn` name only when no device ID can be read.
- Mixed DPI: create a window at a position on the target monitor first, then size it.
- A taskbar left hidden after a crash must always be restored (guard process and start-up recovery), also for secondary taskbars.
- Commit messages: lowercase `feat:`/`fix:`/`chore:` style of the repo, no co-author or tool lines. Texts in English and Turkish (`src/locales/en.json`, `tr.json`).
- Checks before every commit: `./node_modules/.bin/tsc.exe --noEmit` (empty output), `node --test src/*.test.ts src/*/*.test.ts`, `cd src-tauri && cargo test --lib`.
- Builds for trying: the user runs `npm run fast`; do not start builds on your own, ask at each **Phase gate**.

## Review Focus

1. A monitor is unplugged while it has bar windows: the planner destroys exactly those windows, the other monitors' bars are untouched (test in Task 4).
2. The user makes another monitor the primary one: the static pair moves to the new primary and the dynamic pair that was on it is destroyed (test in Task 4).
3. A saved monitor entry for a monitor that is not plugged in stays in the settings, and two entries with the same key collapse to one (test in Task 3).
4. A fullscreen app on monitor A does not hide or reveal monitor B's bars (test in Task 10).
5. A crash with secondary taskbars hidden: the saved state tells the guard and the next start to show them again (test in Task 11).

## File Structure

- Create `src-tauri/src/bar.rs`: label parsing/building, `bar_labels(app)`. One responsibility: what a bar window label means.
- Create `src-tauri/src/monitors.rs`: `MonitorInfo`, key choice, `desired`, `plan`, listing, applying, polling, `get_monitors`, `is_enabled`, `place`.
- Create `src/bar.ts` (+ `src/bar.test.ts`): `barKind(label)`.
- Create `src/monitors.ts` (+ `src/monitors.test.ts`): card model for the settings section. Create `src/settings/MonitorsSection.tsx`.
- Modify: `lib.rs`, `settings.rs`, `hit_region.rs`, `visibility.rs`, `fullscreen.rs`, `hotkeys.rs`, `preview.rs`, `app_catalog.rs`, `dock_menu.rs`, `taskbar.rs`, `taskbar_plan.rs`, `src/main.tsx`, `src/hitRegion.ts`, `src/bridge.ts`, `src/settings/IslandSettings.tsx`, locales, `CHANGELOG.md`.

---

## Phase 1: windows and settings

### Task 1: Bar labels (`bar.rs`, `bar.ts`)

**Files:** Create `src-tauri/src/bar.rs`, `src/bar.ts`, `src/bar.test.ts`; Modify `src-tauri/src/lib.rs` (add `mod bar;`).

**Interfaces:**
- Produces (Rust): `pub enum BarKind { Notch, Dock }` (`Debug, Clone, Copy, PartialEq, Eq, Hash`), `pub fn parse(label: &str) -> Option<(BarKind, Option<u32>)>`, `pub fn label(kind: BarKind, n: Option<u32>) -> String`, `pub fn kind_of(label: &str) -> Option<BarKind>`, `pub fn is_bar(label: &str) -> bool`, `pub fn bar_labels(app: &tauri::AppHandle) -> Vec<String>` (labels of all existing webview windows that are bars, sorted).
- Produces (TS): `export function barKind(label: string): "notch" | "dock" | null`.

- [ ] **Step 1: Write the failing tests** (`bar.rs` tests, `src/bar.test.ts`): `parse("notch") == Some((Notch, None))`, `parse("dock-2") == Some((Dock, Some(2)))`, `parse("notch-") == None`, `parse("notch-x") == None`, `parse("docker") == None`, `parse("settings") == None`, `label(Dock, Some(3)) == "dock-3"`, `label(Notch, None) == "notch"`, `parse(&label(k, n)) == Some((k, n))`. TS: `barKind("dock-2") === "dock"`, `barKind("preview") === null`.
- [ ] **Step 2: Run and see them fail:** `cd src-tauri && cargo test --lib bar::` and `node --test src/bar.test.ts` (functions not defined).
- [ ] **Step 3: Implement** both files; a suffix counts only when it is `-` followed by digits.
- [ ] **Step 4: Run both again, expect PASS.** Commit: `feat: bar window labels (notch, dock-2) in one place`.

### Task 2: Retarget the hard-coded labels (no behaviour change yet)

**Files:** Modify `hit_region.rs`, `visibility.rs`, `src/main.tsx`, `src/hitRegion.ts`, `src/notch/DynamicIsland.tsx` (the `useHitRegion` call), `src/dock/Dock.tsx` (same), `src/bridge.ts` (`setHitRegion`/`clearHitRegion` label type becomes `string`).

**Interfaces:**
- Consumes: Task 1.
- Produces: `hit_region::is_bar(label)` = `bar::is_bar`; `useHitRegion(box: Box | null, radius: number)` (label taken from `getCurrentWindow().label`); in `visibility.rs` the loop works on `bar::bar_labels(&app)` with a `HashMap<String, Option<bool>>` for `shown`, the visibility mode picked by `bar::kind_of(label)`; `hidden_in_fullscreen`, `overlap_probe`, `reveal_band` and the `held` rule use `kind_of` instead of `label == "notch"`.

- [ ] **Step 1: Test first:** in `hit_region.rs` change `only_the_two_bars_can_be_shaped` to also assert `is_bar("dock-2")` and `!is_bar("preview")`; existing `visibility` tests (`hidden_in_fullscreen("notch", ...)` etc.) must keep passing unchanged.
- [ ] **Step 2: Run:** `cargo test --lib hit_region visibility` (the new assertion fails).
- [ ] **Step 3: Implement** the retargeting; `main.tsx` chooses the island view for `barKind(label) === "notch"` and the dock view for `"dock"`, other labels as before. The hit-region polling loop in `hit_region::start` iterates `bar_labels` each tick.
- [ ] **Step 4: Run all checks** (Global Constraints). With one monitor the app must behave exactly as before.
- [ ] **Step 5: Commit:** `refactor: bars are found by label kind instead of the two fixed labels`.

### Task 3: `monitors` setting

**Files:** Modify `settings.rs`, `src/bridge.ts`, `src/notch/islandDefaults.ts` (only if `Settings` defaults are mirrored there; otherwise none).

**Interfaces:**
- Produces: `#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)] pub struct MonitorCfg { pub key: String, pub name: String, pub island: bool, pub dock: bool }`; field `pub monitors: Vec<MonitorCfg>` on `Settings` (default empty); `Settings::normalize_monitors(&mut self)` dropping later entries with a key already seen; called from `set_setting` after `updated.island.normalize()`. TS: `export interface MonitorCfg { key: string; name: string; island: boolean; dock: boolean }` and `monitors: MonitorCfg[]` on `Settings`.

- [ ] **Step 1: Failing tests:** an old settings JSON without `monitors` loads with `monitors == []`; a list round-trips through `to_json`/`from_json`; `normalize_monitors` on `[a(island:true), a(island:false), b]` leaves `[a(island:true), b]` (Review Focus 3).
- [ ] **Step 2-4:** run, implement, run (`cargo test --lib settings`). 
- [ ] **Step 5: Commit:** `feat: a monitors list in the settings (which monitors have an island and a dock)`.

### Task 4: Monitor planner (pure part of `monitors.rs`)

**Files:** Create `src-tauri/src/monitors.rs` (pure part first); `lib.rs` gets `mod monitors;`.

**Interfaces:**
- Consumes: `bar::BarKind`, `settings::MonitorCfg`, `fullscreen::Rect` (`left, top, right, bottom: i32`).
- Produces:
  - `pub struct MonitorInfo { pub key: String, pub name: String, pub rect: Rect, pub work: Rect, pub scale: f64, pub primary: bool }`
  - `pub fn monitor_key(device_id: Option<&str>, name: &str) -> String`
  - `pub struct Want { pub key: String, pub island: bool, pub dock: bool }`
  - `pub fn desired(monitors: &[MonitorInfo], cfg: &[MonitorCfg]) -> Vec<Want>` (one per live monitor; saved entry wins; no entry: primary `true/true`, others `false/false`)
  - `pub struct BarWin { pub label: String, pub kind: BarKind, pub key: String }` (an existing bar window and the monitor it is on)
  - `pub enum Action { Create { kind: BarKind, key: String }, Destroy { label: String } }`
  - `pub fn plan(monitors: &[MonitorInfo], want: &[Want], existing: &[BarWin]) -> Vec<Action>`: for every non-primary monitor and kind that is wanted and has no window, `Create`; every existing window with a numbered label (`dock-2`, `notch-3`) whose monitor is gone, is no longer wanted, or is now the primary monitor, `Destroy`; windows labelled `notch`/`dock` are never in the result.
  - `pub fn is_wanted(want: &[Want], key: &str, kind: BarKind) -> bool`

- [ ] **Step 1: Failing tests** (names and assertions): `key_prefers_the_device_id` (`monitor_key(Some("MONITOR\\DEL4012"), "\\\\.\\DISPLAY2") == "monitor\\del4012"`; `None` and `Some("")` give the name); `defaults_are_primary_on_others_off`; `a_saved_entry_wins`; `a_saved_monitor_that_is_not_plugged_in_is_ignored`; `creates_windows_for_a_wanted_second_monitor` (two `Create`); `nothing_to_do_when_windows_exist`; `unplugged_monitor_destroys_only_its_windows` (existing `dock-2` on a missing key and `dock-3` on a live one: only `Destroy dock-2`) (Review Focus 1); `new_primary_destroys_the_dynamic_pair_on_it` (existing `notch-2` whose key is now primary: `Destroy notch-2`, and no `Create` for the old primary) (Review Focus 2); `static_windows_are_never_touched`.
- [ ] **Step 2-4:** run (`cargo test --lib monitors::`), implement, run.
- [ ] **Step 5: Commit:** `feat: the monitor planner (which bar windows should exist)`.

### Task 5: Monitors on Windows (list, apply, place, poll)

**Files:** Modify `src-tauri/src/monitors.rs`, `lib.rs` (replace `place_notch`/`place_dock`, call `monitors::start` in `setup`, register `get_monitors`, handle the `"monitors"` key in `set_setting`) (no `Cargo.toml` change expected: `EnumDisplayDevicesW` is in `Win32_Graphics_Gdi`, already enabled).

**Interfaces:**
- Consumes: Tasks 1, 3, 4.
- Produces:
  - `pub fn list(app: &AppHandle) -> Vec<MonitorInfo>` (from `app.available_monitors()`; key from the device ID via `EnumDisplayDevicesW`, fallback name; `primary` from `app.primary_monitor()`).
  - `pub fn place(app: &AppHandle, label: &str)`: the island centered on the top edge of its monitor, the dock on the bottom of its work area (the formulas of `place_notch`/`place_dock`, on that bar's monitor; the monitor of a bar comes from a `static BAR_MONITOR: Mutex<HashMap<String, String>>` label → key, static pair mapped to the primary key).
  - `pub fn reconcile(app: &AppHandle)`: `desired` → `plan` → apply (`WebviewWindowBuilder` with label `bar::label(kind, Some(n))`, the options of `tauri.conf.json`, `windows_list::register_bar`, position first, then size), `place` for every bar, then emits `monitors-changed`.
  - `pub fn start(app: AppHandle)`: calls `reconcile`, then a thread comparing a signature (keys, rects, work areas, scales) every 2 s and calling `reconcile` on a change.
  - `pub fn is_enabled(app: &AppHandle, label: &str) -> bool` (uses `is_wanted`; a label that is not a bar: `true`).
  - `#[tauri::command] pub fn get_monitors(app, state) -> Vec<MonitorView>` with `MonitorView { key, name, width, height, x, y, primary, island, dock }`.

- [ ] **Step 1:** write unit tests only for what is pure here: the signature function `signature(&[MonitorInfo]) -> String` changes when a rect, a work area or a scale changes and not otherwise.
- [ ] **Step 2-4:** implement, `cargo test --lib monitors::`, `tsc` etc.
- [ ] **Step 5: Commit:** `feat: bar windows on every chosen monitor (created, moved and removed as monitors change)`.

### Task 6: Visibility respects "disabled"

**Files:** Modify `visibility.rs`.

**Interfaces:** Consumes `monitors::is_enabled`. Produces: in the loop, `want = enabled && (...)`; a disabled bar is hidden and stays hidden.

- [ ] **Step 1: Failing test** on a pure helper `pub fn final_want(enabled: bool, wanted: bool) -> bool` (`false` when not enabled, else `wanted`) in `visibility.rs`, with the four combinations.
- [ ] **Step 2-4:** run, implement and use it in the loop, run all Rust tests.
- [ ] **Step 5: Commit:** `feat: a bar of a switched-off monitor stays hidden`.

### Task 7: Monitors section in Settings

**Files:** Create `src/monitors.ts`, `src/monitors.test.ts`, `src/settings/MonitorsSection.tsx`; Modify `src/bridge.ts` (`getMonitors`, `MonitorView`), `src/settings/IslandSettings.tsx` (section below Layout), `src/settings/Settings.css`, both locales.

**Interfaces:**
- Produces (`src/monitors.ts`): `export interface MonitorCard { key: string; name: string; width: number; height: number; x: number; y: number; primary: boolean; island: boolean; dock: boolean; connected: boolean }`; `export function monitorCards(live: MonitorView[], saved: MonitorCfg[]): MonitorCard[]` (live monitors in position order, then saved ones that are not plugged in, marked `connected: false`); `export function setFlag(saved: MonitorCfg[], card: MonitorCard, flag: "island" | "dock", on: boolean): MonitorCfg[]` (adds or updates the entry for that key, keeps the other flag's current value); `export function showSection(cards: MonitorCard[]): boolean` (`true` only with two or more cards).

- [ ] **Step 1: Failing tests:** `monitorCards` orders by `x`; keeps an unplugged saved monitor last with `connected: false`; `setFlag` creates an entry that keeps the card's other flag; `showSection([one card]) === false`.
- [ ] **Step 2-4:** run (`node --test src/monitors.test.ts`), implement, then the component: one tile per card in the style of the island type tiles (`island-style-tile`), a small monitor drawing placed by position, name and "1920x1080, primary", two `Toggle`s (Island, Dock), changes through `setSetting("monitors", ...)`.
- [ ] **Step 5: Commit:** `feat: a Monitors section in Settings (island and dock switches per monitor)`.

**Phase gate 1:** `tsc`, all tests green; ask the user to run `npm run fast` and check on the real two monitors: second monitor on, island and dock appear and look right (sizes, mixed DPI), off again, unplug/replug. Fix what they find before Phase 2.

---

## Phase 2: panels and shortcuts

### Task 8: Panels open on the calling bar's monitor

**Files:** Modify `lib.rs` (`toggle_tray_panel`, `island_focus`), `preview.rs` (`show_preview`), `app_catalog.rs`, `dock_menu.rs` (the commands at lines using `get_webview_window("dock")`).

**Interfaces:** each of these `#[tauri::command]`s gets a `window: tauri::WebviewWindow` parameter (Tauri injects it) and uses it where the code looked up `"dock"`/`"notch"`. `grep -n 'get_webview_window("dock")\|get_webview_window("notch")' src-tauri/src` must list only the places that really mean the primary bar (setup registration, `place` for the static pair).

- [ ] **Step 1:** no pure logic to test; write down the grep result before and after in the commit body.
- [ ] **Step 2:** implement; `cargo test --lib`, `tsc`.
- [ ] **Step 3: Commit:** `feat: preview, tray, app catalog, dock menu and island focus use the bar that opened them`.

### Task 9: Win+number goes to the dock under the pointer

**Files:** Modify `hotkeys.rs`, `monitors.rs`.

**Interfaces:** `pub fn pick_dock(docks: &[(String, Rect)], point: (i32, i32)) -> Option<String>` in `monitors.rs` (the label of the dock window whose monitor rect contains the point; `None` when none does); `hotkeys.rs` emits `dock-shortcut` to `pick_dock(...)` or `"dock"`.

- [ ] **Step 1: Failing tests:** point on monitor 2 picks `dock-2`; point outside every monitor gives `None`; the primary dock label `"dock"` is picked for a point on the primary monitor.
- [ ] **Step 2-4:** run, implement, run.
- [ ] **Step 5: Commit:** `feat: the Win+number shortcut goes to the dock on the monitor under the pointer`.

**Phase gate 2:** user tries previews, hidden-icons panel, add-app panel, dock menu and Win+number on the second dock.

---

## Phase 3: fullscreen per monitor

### Task 10: `fullscreen::state_for`

**Files:** Modify `fullscreen.rs`, `visibility.rs`.

**Interfaces:** `pub fn covers_for(fg: Option<(Rect, bool)>, monitor: Rect) -> Option<bool>` (pure: `None` when `fg` is `None`; the stored `covers` when `fg`'s monitor rect equals `monitor`; `Some(false)` for any other monitor); `fullscreen::probe` stores `(monitor rect, covers)`; `pub fn state_for_window(hwnd: HWND) -> Option<bool>` uses `MonitorFromWindow` and `covers_for`. `visibility.rs` uses `state_for_window(bar hwnd)` instead of `state()`. The `fullscreen-changed` event keeps its payload (nothing in the frontend listens to it besides `onFullscreenChanged` in `bridge.ts`).

- [ ] **Step 1: Failing tests:** `covers_for(Some((a, true)), a) == Some(true)`; `covers_for(Some((a, true)), b) == Some(false)` (Review Focus 4); `covers_for(None, a) == None`; `covers_for(Some((a, false)), a) == Some(false)`.
- [ ] **Step 2-4:** run, implement, run all Rust tests.
- [ ] **Step 5: Commit:** `feat: a fullscreen app only affects the bars of its own monitor`.

**Phase gate 3:** user plays a fullscreen video on one monitor: that monitor's bars follow their visibility mode, the other monitor's bars stay.

---

## Phase 4: the second Windows taskbar

### Task 11: Plan and state for secondary taskbars

**Files:** Modify `taskbar_plan.rs` (including the existing test literals of `TaskbarState`).

**Interfaces:** `TaskbarState` gets `#[serde(default)] pub hidden_secondary: bool` (`STATE_VERSION` stays 1, old files load with `false`); `Backend` gets `fn hide_secondary(&self, monitors: &[Rect])` (hides the secondary taskbar windows that sit on one of these monitors), `fn show_secondary(&self)` (shows every hidden secondary taskbar window), `fn secondary_hidden(&self) -> bool`; `plan_hide(existing, current_autohide, chevron_present, pid, hide_secondary: bool)` sets `state_to_save.hidden_secondary`; `RestorePlan` gets `show_secondary: bool` (true when the state says so, or when there is no state and `secondary_hidden`); `guard_should_restore` and `needs_stale_restore` also fire for a state with `hidden_secondary`.

- [ ] **Step 1: Failing tests** with `FakeBackend`/`MemStore` extended: a hide with `hide_secondary: true` saves `hidden_secondary: true` and calls `hide_secondary`; `restore` shows them and clears the state; an old state JSON without the field loads with `false`; a stale state (app dead) with `hidden_secondary` makes `needs_stale_restore` true (Review Focus 5); no state but a hidden secondary taskbar: `needs_final_show`-style recovery shows it.
- [ ] **Step 2-4:** run (`cargo test --lib taskbar_plan`), implement, fix the other tests' literals and the callers of `plan_hide`/`hide` in `taskbar.rs` (they pass `false` until Task 12), run.
- [ ] **Step 5: Commit:** `feat: the taskbar plan knows about secondary taskbars`.

### Task 12: Windows side, restore paths and reconcile hook

**Files:** Modify `taskbar.rs` (`WinBackend` implements the three new methods with `EnumWindows` over class `Shell_SecondaryTrayWnd`, matching a window to a monitor by its rect centre), `lib.rs` (`apply_replace_taskbar` passes the monitors that have a dock; `monitors::reconcile` calls it again when that set changes), guard code path and `restore_now` (already go through `plan_restore`).

- [ ] **Step 1:** no new pure logic; extend an existing `taskbar.rs` test only if one exists for the guard path.
- [ ] **Step 2:** implement; `cargo test --lib`, `tsc`.
- [ ] **Step 3: Commit:** `feat: the Windows taskbar of a monitor with a dock is hidden and given back`.

**Phase gate 4:** user switches the second dock on (its Windows taskbar hides), off (it returns), quits MacDows (both return), and kills MacDows from Task Manager (the guard brings both back).

### Task 13: Changelog

**Files:** Modify `CHANGELOG.md` (`[Unreleased]`, English and Türkçe): "An island and a dock on every monitor you choose (Settings > Appearance > Monitors)", fullscreen per monitor, second Windows taskbar. 

- [ ] **Step 1:** add the lines; **Step 2: Commit:** `docs: changelog for multi-monitor support`.
