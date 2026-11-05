# Status Widgets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The island's status pill shows weather, battery, CPU, RAM, disk and network widgets on either side of the clock while hovered, and Settings lets the user place them by drag and drop.

**Architecture:** Settings (`island.screens.status`) gain a `widgets` layout plus weather options, normalized in Rust. One Rust command `get_system_stats` samples CPU/RAM/disk/network; the front end polls it every 2 s only while the island is hovered and a stats widget is placed. Weather is fetched from Open-Meteo by the front end (`fetch`, 30 min `localStorage` cache). All decisions (layout rules, width, formatting, icon choice) are pure functions in `widgetLogic.ts` with `node --test` tests; the drag-and-drop editor uses pointer events.

**Tech Stack:** Rust (Tauri 2, `windows` 0.61, serde), React 19 + TypeScript, framer-motion, `cargo test`, `node --test src/notch/*.test.ts`.

**Spec:** `docs/superpowers/specs/2026-11-04-status-widgets-design.md` (written 2026-11-04; user decisions in §1).

## Global Constraints

- Widget ids exactly `weather`, `battery`, `cpu`, `ram`, `disk`, `net`; default layout `{ left: ["weather"], right: ["battery"] }`; at most **3** per side; one widget never in two places.
- New `StatusCfg` keys and defaults: `widgets`, `weather_city: ""`, `weather_lat: 0.0`, `weather_lon: 0.0`, `temp_unit: "celsius"` (`"celsius" | "fahrenheit"`), `low_battery: 20` (5-50, step 5). `battery_percent` is removed from the model but still read from old files (migration, spec §5).
- Sampling only while hovered and at least one of cpu/ram/disk/net placed; interval 2000 ms; nothing sampled when the island is not hovered.
- Weather: no request while `weather_city` is empty (no IP lookup); only city name and coordinates leave the machine; Open-Meteo endpoints `https://geocoding-api.open-meteo.com/v1/search?name=<q>&count=5` and `https://api.open-meteo.com/v1/forecast?latitude=<lat>&longitude=<lon>&current=temperature_2m,weather_code`; cache key `macdows.weather`, 30 min.
- No new Rust or npm dependencies. Network rate display `↓ 1,2 MB/s`, below 1 KB/s `0`. Text 12 px. Width: base + 54 px per widget (net widget 84 px).
- Drag uses `pointerdown/move/up` + `setPointerCapture`, never HTML5 drag-and-drop; keyboard: `Alt+←/→` change side, `Alt+↑/↓` reorder.
- Icons are MacDows' own drawings (no SF Symbols). All strings in `src/locales/en.json` and `tr.json` under `widget.*`.
- Commit style: lowercase conventional (`feat:`, `fix:`), author Fatal-IV, **no Claude/co-author lines**; detailed `CHANGELOG.md` entry in the last task.
- Real-PC verification: stop `macdows`, `bun run tauri build --no-bundle`, run it, hover the island, screenshot only the top strip.

## Review Focus

- Old settings file with `battery_percent: false` and no `widgets` → no battery widget, nothing else changes. (Task 1)
- Hand-edited layout with unknown ids, duplicates, or 5 items on one side → repaired to the rules, never a crash. (Task 1)
- Desktop PC with no battery and `battery` placed → widget hidden, layout width does not reserve space for it. (Tasks 3, 4)
- Offline machine, or city typed but Open-Meteo unreachable → last cached value, else the weather widget is hidden; no console error loop. (Task 4)
- Network counter goes backwards (adapter reset) or two samples with zero elapsed time → rate 0, no negative or NaN. (Task 2)
- Dragging a chip onto a full side → rejected, side flashes red, layout unchanged. (Task 5)

---

## File Structure

- `src-tauri/src/island_settings.rs` — modify: `StatusCfg` fields, `Widgets` struct, normalize, migration.
- `src-tauri/src/system_stats.rs` — create: `get_system_stats` and pure diff helpers.
- `src-tauri/src/lib.rs` — modify: `mod system_stats;` and register the command.
- `src/bridge.ts`, `src/notch/islandDefaults.ts` — modify: types, defaults, `getSystemStats`.
- `src/notch/widgetLogic.ts` (+ `widgetLogic.test.ts`) — create: pure rules and formatting.
- `src/notch/StatusWidgets.tsx` (+ CSS in `DynamicIsland.css`) — create: widget row, icons, `useStats`, `useWeather`.
- `src/notch/DynamicIsland.tsx` — modify: render row, width chain, drop old `.battery` span.
- `src/settings/StatusWidgetsEditor.tsx` — create; `src/settings/IslandSettings.tsx` — modify.
- `src/locales/en.json`, `tr.json`, `CHANGELOG.md` — modify.

---

### Task 1: Settings model, normalization, migration

**Files:**
- Modify: `src-tauri/src/island_settings.rs`, `src/bridge.ts`, `src/notch/islandDefaults.ts`
- Test: `src-tauri/src/island_settings.rs` (`mod tests`)

**Interfaces:**
- Produces (Rust): `pub struct Widgets { pub left: Vec<String>, pub right: Vec<String> }` (`Default` = weather left / battery right); `StatusCfg` gains `widgets: Widgets`, `weather_city: String`, `weather_lat: f64`, `weather_lon: f64`, `temp_unit: String`, `low_battery: u8`; `pub const WIDGET_IDS: [&str; 6]`; `Island::normalize` also repairs widgets and clamps `low_battery`.
- Produces (TS): `Widgets`, extended `StatusCfg` in `bridge.ts` (no `battery_percent`); `defaultIsland()` updated.
- Note: `f64` removes `Eq` — drop `Eq` from the derive on `StatusCfg` and on every parent the compiler names (`Screens`, `Island`, …).
- Migration: `StatusCfg` deserializes through a private shadow that still accepts `battery_percent: Option<bool>`; when `widgets` is absent: `Some(true)`/`None` → default layout, `Some(false)` → battery removed from default layout.

- [ ] **Step 1: Failing tests** — `status_defaults_have_weather_left_and_battery_right`; `old_battery_percent_false_removes_the_battery_widget` (JSON `{"battery_percent":false}`, expect `right == []`, `left == ["weather"]`); `old_battery_percent_true_keeps_the_default_layout`; `unknown_duplicate_and_excess_widgets_are_repaired` (left `["cpu","bogus","cpu","ram","disk","net"]`, right `["cpu","battery"]` → left `["cpu","ram","disk"]`, right `["battery"]`); `low_battery_snaps_to_5_50_in_steps_of_5` (3→5, 22→20, 99→50); `bad_temp_unit_falls_back_to_celsius`.
- [ ] **Step 2: Run** `cargo test island_settings` — Expected: FAIL (compile error / assertions).
- [ ] **Step 3: Implement** the structs, shadow deserializer, and normalization (order: keep first occurrence across left then right, drop unknown, cap 3 per side).
- [ ] **Step 4: Mirror in TS** (`bridge.ts`, `islandDefaults.ts`); remove the now-unused `battery_percent` references.
- [ ] **Step 5: Run** `cargo test` and `node node_modules/typescript/bin/tsc --noEmit` — Expected: all pass, tsc clean (fix remaining `battery_percent` usages it reports, e.g. in `DynamicIsland.tsx`, by deleting the condition for now; Task 4 re-adds the battery as a widget).
- [ ] **Step 6: Commit** `feat: status widget settings (layout, weather city, unit, low battery) with migration from battery_percent`

### Task 2: System stats command

**Files:**
- Create: `src-tauri/src/system_stats.rs`
- Modify: `src-tauri/src/lib.rs` (add `mod system_stats;`, register `system_stats::get_system_stats` in `generate_handler!`), `src-tauri/Cargo.toml` (add `Win32_System_SystemInformation` to `windows` features if missing), `src/bridge.ts`
- Test: `src-tauri/src/system_stats.rs` (`mod tests`)

**Interfaces:**
- Produces (Rust): `#[derive(Serialize, Default, Clone)] pub struct SystemStats { pub cpu: u8, pub ram: u8, pub disk: u8, pub net_down: u64, pub net_up: u64 }`; `#[tauri::command] pub fn get_system_stats() -> SystemStats`; pure `fn cpu_percent(prev: Option<(u64, u64, u64)>, now: (u64, u64, u64)) -> u8` (tuples are idle, kernel, user; kernel includes idle); pure `fn rate(prev: u64, now: u64, elapsed_ms: u64) -> u64` (bytes per second).
- Produces (TS): `SystemStats` interface and `getSystemStats = () => invoke<SystemStats>("get_system_stats")`.

- [ ] **Step 1: Failing tests** — `cpu_is_zero_without_a_previous_sample`; `cpu_percent_is_busy_over_total` (prev `(100,200,100)`, now `(150,300,200)`: total delta = kernel delta 100 + user delta 100 = 200, idle delta 50, so `100 * (200 - 50) / 200 == 75`); `cpu_percent_stays_within_0_100`; `rate_is_bytes_per_second` (`rate(1000, 3000, 2000) == 1000`); `rate_is_zero_when_the_counter_goes_backwards`; `rate_is_zero_for_zero_elapsed_time`.
- [ ] **Step 2: Run** `cargo test system_stats` — Expected: FAIL (module missing).
- [ ] **Step 3: Implement** the helpers, then `get_system_stats`: CPU via `GetSystemTimes`, RAM `GlobalMemoryStatusEx().dwMemoryLoad`, disk used % of the system drive via `GetDiskFreeSpaceExW` (drive from `%SystemDrive%`, default `C:\`), network = sum of `InOctets`/`OutOctets` over `GetIfTable2` rows with `OperStatus == IfOperStatusUp` and `Type != IF_TYPE_SOFTWARE_LOOPBACK`; previous samples (times + counters + `Instant`) in one `static Mutex<Option<Prev>>`.
- [ ] **Step 4: Run** `cargo test` — Expected: pass. Then `cargo build` compiles with the command registered.
- [ ] **Step 5: Commit** `feat: get_system_stats (cpu, ram, disk, network rates)`

### Task 3: Widget logic (pure TS)

**Files:**
- Create: `src/notch/widgetLogic.ts`
- Test: `src/notch/widgetLogic.test.ts`

**Interfaces:**
- Consumes: `Widgets`, `StatusCfg` (Task 1).
- Produces: `type WidgetId = "weather" | "battery" | "cpu" | "ram" | "disk" | "net"`; `WIDGET_IDS: WidgetId[]`; `visibleWidgets(w: Widgets, ctx: { hasBattery: boolean; weatherReady: boolean }): { left: WidgetId[]; right: WidgetId[] }` (drops battery without battery, weather when not ready); `widgetsWidth(v: { left: WidgetId[]; right: WidgetId[] }): number` (54 per widget, 84 for `net`); `needsStats(v): boolean`; `formatRate(bytesPerSecond: number): string`; `formatTemp(celsius: number, unit: "celsius" | "fahrenheit"): string` (`13°`, rounded, °F converted); `weatherIcon(code: number): "clear" | "partly" | "cloudy" | "fog" | "rain" | "snow" | "storm"`; `isLowBattery(percent: number, threshold: number, charging: boolean): boolean`; `moveWidget(w: Widgets, id: WidgetId, to: { side: "left" | "right" | "tray"; index: number }): Widgets | null` (null when target side already holds 3; removes from the old place first); `trayWidgets(w: Widgets): WidgetId[]`.

- [ ] **Step 1: Failing tests** — `formatRate`: 0 → `0`, 900 → `0`, 1_258_291 → `↓ 1,2 MB/s`, 52_000 → `↓ 51 KB/s`; `formatTemp(13.4,"celsius") === "13°"`, `formatTemp(0,"fahrenheit") === "32°"`, `formatTemp(-0.4,"celsius") === "0°"`; `weatherIcon` table: 0→clear, 2→partly, 3→cloudy, 45→fog, 63→rain, 73→snow, 95→storm, unknown 999→cloudy; `visibleWidgets` hides battery when `hasBattery` false and weather when not ready; `widgetsWidth` = 54·n with net counted 84; `moveWidget` returns null for a 4th item on a side, moves between sides, to tray, and reorders inside a side; `isLowBattery(15,20,false)` true, `(15,20,true)` false, `(20,20,false)` false.
- [ ] **Step 2: Run** `node --test src/notch/widgetLogic.test.ts` — Expected: FAIL (module missing).
- [ ] **Step 3: Implement** the functions above (decimal comma in `formatRate` uses `toFixed(1).replace(".", ",")`; MB when ≥ 1 MiB, KB when ≥ 1 KiB).
- [ ] **Step 4: Run** the same command and `node node_modules/typescript/bin/tsc --noEmit` — Expected: pass, clean.
- [ ] **Step 5: Commit** `feat: widget logic (layout rules, formatting, weather icons)`

### Task 4: Widget row in the island

**Files:**
- Create: `src/notch/StatusWidgets.tsx`
- Modify: `src/notch/DynamicIsland.tsx` (width at line ~519, the old `.battery` span at ~757, render the two sides around the clock), `src/notch/DynamicIsland.css`, `src/notch/icons.tsx` (widget icons), `src/locales/en.json`, `tr.json`

**Interfaces:**
- Consumes: `visibleWidgets`, `widgetsWidth`, `needsStats`, `formatRate`, `formatTemp`, `weatherIcon`, `isLowBattery` (Task 3); `getSystemStats` (Task 2); `useSystemStatus()` (existing, has `battery_percent`, may be null).
- Produces: `export function StatusWidgets({ side, ids, stats, weather, status, cfg }: …): JSX.Element` (one side's row); `export function useStats(active: boolean): SystemStats | null` (polls every 2000 ms while `active`, clears on false); `export function useWeather(cfg: StatusCfg): { temp: number; code: number } | null` (empty city → `null` and no request; coordinates always come from settings, no geocoding here; fetches the forecast, caches 30 min under `macdows.weather`, serves the cache when the fetch fails).

- [ ] **Step 1: Failing test** — add to `widgetLogic.test.ts` a `weatherCacheFresh(savedAtMs, nowMs): boolean` case (`fresh` under 30 min, stale at 30 min) and implement `weatherCacheFresh` in `widgetLogic.ts` first; run it red then green.
- [ ] **Step 2: Implement** `StatusWidgets.tsx`: hooks plus the row component; low battery uses a red tone class; hidden widgets render nothing.
- [ ] **Step 3: Integrate** in `DynamicIsland.tsx`: `useStats(hovered && needsStats(visible))`; compute `visible` from `island.screens.status.widgets`; hovered status width becomes `170 + widgetsWidth(visible)` (replaces the `battery != null ? 200 : 170` branch); render left row before the clock and right row after; remove the old `.battery` span and its CSS.
- [ ] **Step 4: Verify** `node node_modules/typescript/bin/tsc --noEmit` clean and `node --test src/notch/*.test.ts` passes; then build release, hover the island on this PC, screenshot: weather hidden (no city), battery on the right; edit `settings.json` by hand to place `cpu`, `ram`, `disk`, `net` and compare to Task Manager.
- [ ] **Step 5: Commit** `feat: status widgets on the island pill (weather, battery, cpu, ram, disk, network)`

### Task 5: Settings editor (drag and drop, city, unit, threshold)

**Files:**
- Create: `src/settings/StatusWidgetsEditor.tsx`
- Modify: `src/settings/IslandSettings.tsx` (Status section), `src/settings/Settings.css`, `src/locales/en.json`, `tr.json`

**Interfaces:**
- Consumes: `moveWidget`, `trayWidgets`, `WIDGET_IDS` (Task 3); `setSetting` for `island` (same call the other island rows use); `StatusCfg` (Task 1); `useSystemStatus()` to dim the battery chip when `battery_percent == null`.
- Produces: `export function StatusWidgetsEditor({ cfg, onChange }: { cfg: StatusCfg; onChange: (next: StatusCfg) => void }): JSX.Element`; pure `searchCity(query: string): Promise<{ name: string; admin: string; country: string; lat: number; lon: number }[]>` calling the geocoding endpoint (empty query → `[]`, network error → throws, caller shows it under the box).

- [ ] **Step 1: Implement the editor**: preview row `[left zone] 12:34 [right zone]`, "Available" tray, chips with icon + name; pointer-event drag with `setPointerCapture`, drop target found by `elementFromPoint`, result from `moveWidget` (null → flash the zone red for 400 ms, keep layout); keyboard `Alt+←/→` (side) and `Alt+↑/↓` (order).
- [ ] **Step 2: Add the options**: city box with result list (picking writes `weather_city`, `weather_lat`, `weather_lon`), unit `Select`, low-battery `Slider` (5-50, step 5), Reset button (default layout, leaves city alone), one-sentence privacy note from spec §4; error text under the city box.
- [ ] **Step 3: Verify** tsc clean; build release, open Settings > Dynamic Island, drag chips between zones and the tray, try a 4th chip on a full side, pick Ankara and confirm the island shows the real temperature; screenshot the editor and the pill.
- [ ] **Step 4: Commit** `feat: drag-and-drop status widget editor with city search, unit and low battery threshold`

### Task 6: Strings, changelog, final pass

**Files:**
- Modify: `src/locales/en.json`, `src/locales/tr.json` (any missing `widget.*` keys: names, tray title, battery-less hint "Bu bilgisayarda pil yok" / "No battery on this computer", privacy sentence, errors), `CHANGELOG.md`, `docs/superpowers/specs/2026-11-04-status-widgets-design.md` (status line → implemented)

- [ ] **Step 1: Check** every `t("widget.…")` key used exists in both locale files (`grep -rho 't("widget\.[a-zA-Z.]*"' src | sort -u` against both JSON files); add the missing ones.
- [ ] **Step 2: Changelog**: detailed entry listing each widget, the hover-only sampling, the privacy behavior of weather, and the migration of `battery_percent`.
- [ ] **Step 3: Run** `cargo test`, `node --test src/notch/*.test.ts`, `node node_modules/typescript/bin/tsc --noEmit` — Expected: all green.
- [ ] **Step 4: Commit** `docs: status widgets strings and changelog`
