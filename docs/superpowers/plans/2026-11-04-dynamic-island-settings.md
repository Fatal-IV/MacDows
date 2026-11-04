# Dynamic Island Settings Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A "Dynamic Island" category in Settings where every island screen and each screen's sub-features can be switched on or off, plus navigation and visibility options.

**Architecture:** A nested `island` object in `settings.json`, owned by Rust (`settings.rs`): defaults fill missing keys, `normalize()` enforces the rules (one screen stays on, ranges, order). The frontend writes the whole `island` object through the existing `set_setting`; `settings-changed` already reaches the island window. All "which screen is reachable" decisions live in one pure TypeScript module (`islandLogic.ts`) so `DynamicIsland.tsx` only asks questions of it.

**Tech Stack:** Rust (serde, Tauri 2), React 19 + TypeScript, `cargo test`, `node --test` (Node 26 runs `.ts` tests directly; test files import with `.ts` extensions and use no enums).

**Spec:** `docs/superpowers/specs/2026-11-03-dynamic-island-settings-design.md` (approved 2026-11-04). The user also decided: visibility, scale and fullscreen move to the new page and are removed from General/Bars/Appearance; all four phases are built in order.

## Global Constraints

- Defaults: all screens and sub-features on, except `nav.remember` off, `shelf.persist` false. `status.clock_12h` false, `tools.week_starts_monday` true, `banner_seconds` 5 (range 3–10), `history_max` 100 (allowed 25/50/100), `shelf.max_items` 40, `nav.click` = `smart`.
- Screen ids, in default order: `status`, `music`, `tools`, `quick`, `notifications`, `shelf`.
- At least one screen is always on. The last one cannot be switched off (UI row is dimmed with the reason; Rust `normalize()` also re-enables `status` if all are off).
- Out of scope: second clock, calendar sync, weather, monitor selection, Nectar's Control Center layout customization.
- Commit messages: lowercase conventional style (`feat:`, `fix:`), author Fatal-IV, **no Claude/co-author lines** (user memory `feedback-commit-attribution`). Settings texts exist in both `src/locales/en.json` and `src/locales/tr.json`.
- Every new key in `settings.json` must load from an old file without it (`#[serde(default)]` on every struct).

## Where the spec is silent (interpretations; no feature of the spec is dropped)

1. **Timer sound and timer progress bar are built, as the spec lists them.** `src/notch/Tools.tsx` has neither today, so Task 6 adds them as new, default-on features (sound: a short chime when the timer reaches zero; progress bar: thin bar under the countdown), each with its switch. Before building, the user is asked how they should look/sound.
2. **Fullscreen** (spec §2.3 "mevcut mantık") becomes `island.fullscreen`: `edge` (today's behavior: shows only when the pointer reaches the screen edge; default) or `hidden` (never shown in fullscreen).
3. **Scale** stays stored in `localStorage["nectar-scale"]` (the island and Appearance already sync it); only its slider moves to the new page.
4. **Muted apps** (`notifications.muted_apps`) suppress the banner only; the notification is still recorded in history.
5. **Status off** removes the status pill from wheel navigation and hides the clock/battery text; the collapsed notch shell itself stays.
6. **Wheel order** now follows `nav.order`; the old behavior of putting Music before Status while media plays is dropped. The hover resting pill still prefers Music while media plays.

## Review Focus

- Old `settings.json` with `notch_visibility: "peek"` and no `island` key → island visibility is `peek` after load, file keeps working. (Task 1)
- `island` in the file with a wrong type/unknown screen in `nav.order`, duplicates, or missing ids → order repaired to a full permutation. (Task 1)
- Frontend sends all screens off → Rust keeps `status` on. (Task 1)
- Only one screen on and it is `tools`/`shelf`/`notifications`: collapsed notch still renders; wheel and click do nothing harmful. (Tasks 2, 5)
- Music screen off while media plays: no player, no auto-switch, click falls through to Quick settings. (Tasks 2, 5)
- Notification history turned off then on again: history starts empty, no crash on missing file. (Task 8)

---

## File Structure

- `src-tauri/src/settings.rs` — modify: `Island` model, defaults, migration, `normalize()`.
- `src-tauri/src/island_settings.rs` — not created; the model stays in `settings.rs` (it is ~150 lines of plain structs).
- `src-tauri/src/lib.rs` — modify: normalize after `set_setting`.
- `src-tauri/src/visibility.rs`, `fullscreen.rs` — modify: read `island.visibility`, `island.fullscreen`.
- `src-tauri/src/notifications.rs`, `shelf.rs` — modify: history flag/cap, shelf persist/max.
- `src/bridge.ts` — modify: `Island` types, remove `notch_visibility`.
- `src/notch/islandLogic.ts` (+ `islandLogic.test.ts`) — create: pure navigation logic.
- `src/settings/IslandSettings.tsx` — create: the whole category page (kept out of `Settings.tsx`).
- `src/settings/Settings.tsx`, `Appearance.tsx` — modify: category entry, remove moved rows.
- `src/notch/DynamicIsland.tsx`, `QuickSettings.tsx`, `Tools.tsx`, `NotificationBanner.tsx`, `Shelf.tsx` — modify: honor settings.
- `src/locales/en.json`, `tr.json` — modify: strings.

---

### Task 1: Island settings model, migration, normalization (phase 1, Rust)

**Files:**
- Modify: `src-tauri/src/settings.rs`, `src-tauri/src/lib.rs:26-50`, `src-tauri/src/visibility.rs:124-135`
- Test: `src-tauri/src/settings.rs` (existing `mod tests`)

**Interfaces:**
- Produces (Rust, all `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]` + `#[serde(default)]`): `Island { visibility: Visibility, fullscreen: FullscreenMode, nav: Nav, screens: Screens }`; `FullscreenMode { Edge, Hidden }` (lowercase serde); `ClickMode { Smart, Quick, Player }` (lowercase); `Nav { wheel: bool, order: Vec<String>, click: ClickMode, middle: bool, remember: bool }`; `Screens { status: StatusCfg, music: MusicCfg, tools: ToolsCfg, quick: QuickCfg, notifications: NotifCfg, shelf: ShelfCfg }` with fields exactly as below.
  - `StatusCfg { enabled, clock_12h, battery_percent }`
  - `MusicCfg { enabled, visualizer, album_glow, progress, auto_open, artwork }`
  - `ToolsCfg { enabled, calendar, week_starts_monday, timer, timer_sound, timer_progress, stopwatch }`
  - `QuickCfg { enabled, wifi, bluetooth, dnd, battery_saver, airplane, hotspot, accessibility, brightness, volume, lock, macdows_settings, windows_settings }`
  - `NotifCfg { enabled, banner, banner_seconds: u32, save_history, history_max: usize, muted_apps: Vec<String> }`
  - `ShelfCfg { enabled, persist, max_items: usize, auto_open }`
- `Settings` gains `pub island: Island`; `notch_visibility` is removed (dock_visibility stays).
- `pub const SCREEN_IDS: [&str; 6]`; `impl Island { pub fn normalize(&mut self) }`.
- `Settings::from_json(&str)` performs the migration: if the parsed JSON object has no `"island"` key and has `"notch_visibility"`, copy that value into `island.visibility`.

- [ ] **Step 1: Write failing tests** in `settings.rs`:
  - `old_notch_visibility_moves_into_the_island`: `from_json(r#"{"language":"tr","notch_visibility":"peek"}"#).island.visibility == Visibility::Peek`.
  - `an_existing_island_object_wins_over_the_old_key`: `{"notch_visibility":"peek","island":{"visibility":"smart"}}` → `Smart`.
  - `a_partial_island_is_filled_with_defaults`: `{"island":{"screens":{"music":{"visualizer":false}}}}` → `visualizer == false`, `music.enabled == true`, `nav.wheel == true`, `notifications.banner_seconds == 5`.
  - `order_is_repaired_to_a_full_permutation`: `normalize()` on order `["shelf","bogus","shelf","music"]` gives `["shelf","music","status","tools","quick","notifications"]`; empty order gives `SCREEN_IDS` order.
  - `the_last_screen_cannot_all_be_off`: set all six `enabled=false`, `normalize()` → `status.enabled == true`.
  - `ranges_are_clamped`: `banner_seconds` 1→3 and 99→10; `history_max` 30→50 (nearest allowed of 25/50/100); `max_items` 0→1.
  - Update the existing round-trip test to use `island` instead of `notch_visibility`.
- [ ] **Step 2: Run** `cd src-tauri && cargo test settings:: ` — Expected: FAIL (compile errors / missing types).
- [ ] **Step 3: Implement** the types above with the defaults from Global Constraints. `from_json` parses to `serde_json::Value` first, applies the migration, then `serde_json::from_value`, then `island.normalize()`; any failure → `Settings::default()`. `load()` keeps its `.bad` rename behavior and also calls `normalize()`.
- [ ] **Step 4: Wire** `set_setting` in `lib.rs`: after `serde_json::from_value`, call `updated.island.normalize()` before saving. Replace both `notch_visibility` uses in `visibility.rs` with `island.visibility`.
- [ ] **Step 5: Run** `cd src-tauri && cargo test` — Expected: all pass (previous 100+ tests included). Then `cargo build` for warnings only.
- [ ] **Step 6: Commit** `feat: Dynamic Island settings model with migration and normalization`.

### Task 2: Pure navigation logic (phase 2 core, TypeScript)

**Files:**
- Create: `src/notch/islandLogic.ts`, `src/notch/islandLogic.test.ts`
- Modify: `src/bridge.ts` (add `Island` and nested interfaces mirroring Task 1 field-for-field, `FullscreenMode`, `ClickMode`, `ScreenId`, `SCREEN_IDS`; `Settings.island: Island`, drop `notch_visibility`), `package.json` (script `"test:logic": "node --test src/notch/islandLogic.test.ts"`)

**Interfaces:**
- Consumes: `Island`, `ScreenId`, `ClickMode` from `bridge.ts`.
- Produces: `export type Stop = "status" | "music" | "player" | "tools" | "quick" | "history" | "shelf"`;
  `buildStops(island: Island, hasMedia: boolean): Stop[]` — walks `nav.order`, skips disabled screens, expands `music` to `["music","player"]` only when `hasMedia`, `notifications` to `["history"]`;
  `clickTarget(island: Island, hasMedia: boolean, playing: boolean): Stop | null` — `smart`: player if `playing && music.enabled`, else quick if enabled; `quick`/`player`: that stop if available; when the wanted stop is unavailable, the first stop from `buildStops` that is not `status`/`music` (pill states); `null` if none;
  `restingStop(island, hasMedia, playing): "status" | "music"` — `music` when `playing && music.enabled`, else `status`;
  `canDisable(island: Island, id: ScreenId): boolean` — false when `id` is the only enabled screen;
  `mutedApp(island: Island, app: string): boolean` (case-insensitive).

- [ ] **Step 1: Write failing tests** (`node:test`, `assert/strict`) with a `base()` default-island helper:
  - `buildStops` default with media: `["status","music","player","tools","quick","history","shelf"]`; without media drops `music` and `player`; music off with media drops both; custom order `["shelf","status",…]` is honored; only `tools` enabled → `["tools"]`.
  - `clickTarget`: smart+playing → `player`; smart+paused → `quick`; smart+playing+music off → `quick`; `quick` off + smart paused → `tools` (first non-pill); only `status` enabled → `null`; `player` mode without media → `quick`.
  - `restingStop`: playing+music on → `music`; playing+music off → `status`.
  - `canDisable`: true with 2 on, false for the last one.
  - `mutedApp(island with muted_apps ["Slack"], "slack") === true`.
- [ ] **Step 2: Run** `bun run test:logic` — Expected: FAIL (module missing).
- [ ] **Step 3: Implement** the functions in `islandLogic.ts` (no React imports, no enums).
- [ ] **Step 4: Run** `bun run test:logic` and `bunx tsc --noEmit` — Expected: tests PASS; tsc reports errors only for `notch_visibility` uses in `Settings.tsx` (fixed in Task 3, so this task also removes that one row's reference by deferring: leave Settings.tsx compiling by reading `s.island.visibility`).
- [ ] **Step 5: Commit** `feat: pure navigation logic for the island screens`.

### Task 3: Settings page skeleton, visibility/scale/fullscreen moved (phase 1, UI)

**Files:**
- Create: `src/settings/IslandSettings.tsx`
- Modify: `src/settings/Settings.tsx` (`Category` union + `CATEGORIES` icon + `ITEMS` entry `island`, placed after `bars`; remove the `notch_visibility` row from the `bars` item), `src/settings/Appearance.tsx` (remove the scale row; export `useStored`), `src/locales/en.json`, `tr.json`
- Modify: `src/notch/DynamicIsland.tsx` (scale reading unchanged; nothing else yet)

**Interfaces:**
- Produces: `export default function IslandSettings({ s, t }: { s: Settings; t: (k: string) => string })` renders sections as `<Group>` blocks; helper `export const setIsland = (patch: (i: Island) => Island) => setSetting("island", patch(structuredClone(current)))`, taking `current` from the `s.island` argument — implemented as `updateIsland(s: Settings, patch: (draft: Island) => void): Promise<unknown>` in the same file, used by all later UI tasks.
- Locale keys: `settings.cat.island` ("Dynamic Island" / "Dinamik Ada"), section titles, `island.visibility`, `island.fullscreen`, `island.fullscreen.edge`, `island.fullscreen.hidden`, `island.scale`, `island.reset`.

- [ ] **Step 1:** Add the category (icon: a rounded pill path) and the page with section "Visibility and layout": visibility `Select` (reuses `vis.*` strings and `settings.visibilityHelp`), fullscreen `Select`, scale `Slider` (same bounds 0.8–1.2 step 0.05 as the old row, via `useStored("nectar-scale","1")`), and a "Reset to defaults" `Button` that restores `visibility`, `fullscreen`.
- [ ] **Step 1b:** Rust: add a failing test in `visibility.rs` that with `FullscreenMode::Hidden` and `fullscreen: Some(true)` the notch is not visible even with `cursor_near: true`; then thread `island.fullscreen` into `Inputs`/`is_visible` for the `notch` label only (the dock keeps today's edge behavior). Run `cargo test visibility::`.
- [ ] **Step 2:** Remove the old notch row and Appearance scale row; keep `settings.dock` where it is.
- [ ] **Step 3:** Run `bunx tsc --noEmit` — Expected: no errors. Run the app (`bun run tauri dev`), open Settings → "Dynamic Island", change visibility to Peek and back; the notch reacts live. Scale slider changes the notch size live.
- [ ] **Step 4: Commit** `feat: Dynamic Island category with visibility, fullscreen and scale`.

### Task 4: Screens and navigation sections in the settings page (phase 2, UI)

**Files:**
- Modify: `src/settings/IslandSettings.tsx`, `src/settings/Settings.css` (small styles for the order list), locales

**Interfaces:**
- Consumes: `updateIsland`, `canDisable` (Task 2).
- Produces: a `ScreenSection` component per screen (`{ id: ScreenId, title, children }`): header row with the screen's `Toggle`; children are shown only while the screen is on; when `!canDisable(island, id)` and the screen is on, the toggle is `disabled` and the row description reads `island.lastScreen` ("At least one screen must stay on"). `OrderList` component: the six screens as rows with up/down buttons (no drag), writes `nav.order`; "Reset" sets default order.
- Navigation section rows: wheel `Toggle`, click `Select` (`smart`/`quick`/`player` with `island.click.*` strings), middle button `Toggle`, remember `Toggle`, order list.

- [ ] **Step 1:** Add the six screen headers (toggles only; sub-feature rows come in Tasks 6–9) and the Navigation section.
- [ ] **Step 2:** `bunx tsc --noEmit` — Expected: clean. In the app, switch screens off until one remains: the last toggle is dimmed with the reason text.
- [ ] **Step 3: Commit** `feat: screen switches and navigation options in settings`.

### Task 5: The island obeys screen and navigation settings (phase 2, runtime)

**Files:**
- Modify: `src/notch/DynamicIsland.tsx`

**Interfaces:**
- Consumes: `buildStops`, `clickTarget`, `restingStop` (Task 2); `useSettings()` from `../i18n`.

- [ ] **Step 1:** Replace `available()`/`stops()` with `buildStops(island, hasMedia)`. `onWheel` returns early when `!island.nav.wheel`. `openQuick` uses `clickTarget` and `goTo`; no target → nothing happens. The middle-button handler returns when `!nav.middle` or `tools` is off.
- [ ] **Step 2:** `restingMode()` uses `restingStop`. When `music` is off, the auto-switch effect (new track → music pill) and the "player" path never run; when `status` is off the pill's clock/battery text is not rendered (empty shell).
- [ ] **Step 3:** `nav.remember`: on hover end, store `currentStop()` in a ref; on the next hover start, if `remember` is on and that stop is still in `buildStops`, call `goTo` on it instead of resting.
- [ ] **Step 4:** Shelf off → the `onDragDropEvent` handler ignores `enter/over/drop` (the `DropZone` never shows). Banner `onOpen`: if `notifications.enabled` is false just dismiss, otherwise open history (spec rule 2).
- [ ] **Step 5: Verify** `bunx tsc --noEmit`, `bun run test:logic`, `cargo test`. Then in the running app, using Settings: (a) turn off Music while a track plays — no player, click opens Quick settings; (b) wheel order follows a reordered list; (c) turn off wheel — wheel does nothing; (d) leave only Tools on — hover works, click opens Tools, no crash; (e) turn on Remember — leaving and re-entering returns to the last screen.
- [ ] **Step 6: Commit** `feat: the island follows screen and navigation settings`.

### Task 6: Status, Music and Tools sub-features (phase 3)

**Files:**
- Modify: `src/notch/DynamicIsland.tsx` (Clock, Visualizer, `ambient-glow`, `Progress`, auto-open effect, `Art`), `src/notch/Tools.tsx` (calendar/timer/stopwatch/week start), `src/settings/IslandSettings.tsx`, locales

**Interfaces:**
- Produces: `Clock({ hour12 }: { hour12: boolean })`; `useTimer()` gains `total: number`; `ToolsPanel` gains prop `cfg: ToolsCfg`; when `calendar`, `timer` or `stopwatch` is off its tab/section is hidden, and if all of timer/stopwatch are off the timer tabs disappear; week start uses `cfg.week_starts_monday` (current code's first-day logic is the Monday default).

- [ ] **Step 1:** Settings rows: Status (clock 24/12 `Select`, battery percent `Toggle`); Music (visualizer, album glow, progress bar, auto-open on new track, artwork); Tools (calendar, week start `Select` Mon/Sun, timer, timer sound, timer progress bar, stopwatch).
- [ ] **Step 2:** Honor each: visualizer hidden in both the pill and the player; `ambient-glow` and `music-gradient` follow "album glow"; `Progress` follows "progress bar"; the auto-open `useEffect` checks `auto_open`; artwork off → show the `art-placeholder` icon instead of the image (also stops `useArtColor` work).
- [ ] **Step 2b:** Build the two new timer features (ask the user first about the chime and bar style): when the countdown hits zero and `timer_sound` is on, play a short chime (Web Audio oscillator, no asset file); with `timer_progress` on, draw a thin bar under the countdown that shrinks from the started duration to zero (`useTimer` exposes `total: number`, the duration last started).
- [ ] **Step 3:** `Clock` uses `toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", hour12 })`.
- [ ] **Step 4: Verify** `bunx tsc --noEmit`; toggle each switch in the running app and watch the island change live without restart.
- [ ] **Step 5: Commit** `feat: status, music and tools sub-feature switches`.

### Task 7: Quick settings sub-features (phase 3)

**Files:**
- Modify: `src/notch/QuickSettings.tsx`, `src/notch/DynamicIsland.tsx` (height of the quick panel), `src/settings/IslandSettings.tsx`, locales

**Interfaces:**
- Consumes: `QuickCfg`. `QuickSettingsScreen` reads `useSettings()?.island.screens.quick` itself.

- [ ] **Step 1:** Filter `ORDER` (wifi, bluetooth, dnd), `SWITCHES` (battery_saver, airplane, hotspot), the accessibility button, the two sliders, and the buttons (lock, MacDows settings, Windows settings) by their flags. A section with nothing left is not rendered (tiles block, sliders block, shortcuts row); the tile grid fills remaining width (CSS grid `auto-fit`; adjust `.quick-tiles` in `DynamicIsland.css`).
- [ ] **Step 2:** Make the panel height follow the visible sections: replace the fixed `312` in `DynamicIsland.tsx` with a value computed from which of the three blocks are visible (tiles ≈ 120, sliders ≈ 80, shortcuts ≈ 52, plus padding; tune against the current 312 when all are on).
- [ ] **Step 3: Verify** with all tiles off the sliders and shortcuts still lay out; with everything off the quick screen shows only the corner arrow (acceptable; the screen switch is the intended way to hide it). Commit `feat: quick settings item switches`.

### Task 8: Notification sub-features (phase 3, Rust + UI)

**Files:**
- Modify: `src-tauri/src/notifications.rs` (history capture at ~165, `HISTORY_MAX`), `src-tauri/src/lib.rs` (command `purge_notification_history` or reuse `clear_notification_history`), `src/notch/DynamicIsland.tsx` (`BANNER_MS`, banner gating), `src/settings/IslandSettings.tsx`, locales

**Interfaces:**
- Consumes: `NotifCfg` read from `AppSettings` state inside the notification poller (`app.state::<AppSettings>()`).
- Produces: `pub fn history_cap(cfg: &NotifCfg) -> usize` (returns `history_max`), used instead of `HISTORY_MAX`; capture is skipped when `!save_history`.

- [ ] **Step 1: Failing test** in `notifications.rs`: `history_is_not_grown_when_saving_is_off` — extract the decision into `fn should_record(cfg: &NotifCfg) -> bool` and assert it is `false` for `save_history=false`; `history_cap` returns 25 for `history_max=25`.
- [ ] **Step 2:** Implement; banner: the `notification-new` handler ignores notices when `!banner`, when the app is in `muted_apps` (`mutedApp`), and uses `banner_seconds * 1000` instead of `BANNER_MS`.
- [ ] **Step 3:** Settings rows: banner `Toggle`, duration `Slider` 3–10, save history `Toggle`, keep `Select` 25/50/100, and the muted-apps list = distinct `app` names from `getNotificationHistory()` plus current `muted_apps`, each with a `Toggle`.
- [ ] **Step 4:** Turning "save history" off asks (an in-page confirmation row with two buttons, not `window.confirm`, per project dialog rule): "Delete saved history too?" Yes → `clearNotificationHistory()`.
- [ ] **Step 5: Verify** `cargo test`, `bunx tsc --noEmit`; send a test notification with banner off (no banner, still in history), with an app muted (no banner), with history off (nothing new recorded). Commit `feat: notification banner and history switches`.

### Task 9: Shelf sub-features (phase 3, Rust + UI)

**Files:**
- Modify: `src-tauri/src/shelf.rs` (`load`, `add_to_shelf`, `MAX_ITEMS`), `src/notch/DynamicIsland.tsx` (drop handler's auto-open), `src/settings/IslandSettings.tsx`, locales

**Interfaces:**
- Consumes: `ShelfCfg`. Pure helper `pub fn keep_stored(persist: bool, stored: &str, current: &str) -> bool` = `persist || same_session(stored, current)`.

- [ ] **Step 1: Failing tests** in `shelf.rs`: `persist_keeps_the_shelf_across_sessions` (`keep_stored(true,"a","b")`), `without_persist_only_the_same_session_stays` (`keep_stored(false,"a","b") == false`, `keep_stored(false,"a","a")`), and `merge` already covers the cap (`merge(..., 1)` keeps one).
- [ ] **Step 2:** Implement: `load()` and `add_to_shelf` read the cfg from `AppHandle` state (commands receive `app: AppHandle`; the TS call signatures do not change). Cap uses `max_items`.
- [ ] **Step 3:** UI: persist `Select` (this session / always), max items `Slider` 10–100 step 10, auto-open on drop `Toggle`; the drop handler only calls `setShelfOpen(true)` when `auto_open`.
- [ ] **Step 4: Verify** `cargo test`; drop a file with auto-open off (file lands, panel stays closed). Commit `feat: shelf persistence, size and auto-open switches`.

### Task 10: Reset buttons, texts, polish (phase 4)

**Files:**
- Modify: `src/settings/IslandSettings.tsx`, locales, `CHANGELOG.md`, `README.md` (settings list, if it mentions the old rows)

- [ ] **Step 1:** "Reset to defaults" under every section: Visibility (Task 3), each screen section (resets only that screen's config but never switches the last screen off), Navigation (including order).
- [ ] **Step 2:** Empty states and copy pass: all `island.*` keys exist in both locales (a quick script `node -e` comparing key sets of `en.json` and `tr.json` for the `island.` and `settings.cat.island` prefixes must print no differences).
- [ ] **Step 3:** CHANGELOG entry under Unreleased. Full verification: `cargo test`, `bun run test:logic`, `bun run build` (tsc + vite) — Expected: all green.
- [ ] **Step 4:** Run the real app once through every section; confirm an old `settings.json` (copy the current file aside first) still loads. Commit `feat: reset buttons and texts for Dynamic Island settings`.

---

## Self-Review

- **Spec coverage:** §2.1 screens → Tasks 6–9 (timer sound/progress built in Task 6); §2.2 nav → Tasks 4–5; §2.3 → Task 3 (+ Rust `fullscreen` use in Task 1/3: `fullscreen.rs`/`visibility.rs` read `island.fullscreen` where `hidden` makes `is_visible` ignore the edge reveal — implemented in Task 3 step 1 together with the Select, with a test in `visibility.rs` that `Inputs{fullscreen:Some(true)}` is hidden when mode is `hidden`); §3 rules 1–8 → Tasks 1, 2, 5, 7, 8, 10; §4 UI → Tasks 3–4; §5 phases → Tasks 1–3, 4–5, 6–9, 10.
- **Type consistency:** `Island` fields identical in Rust (Task 1) and `bridge.ts` (Task 2); `updateIsland` defined in Task 3 and used in 4, 6–10; `ToolsCfg` carries `timer_sound` and `timer_progress` in both places.
- **Open for the user:** interpretations 2–6 above (the spec is silent on them).
