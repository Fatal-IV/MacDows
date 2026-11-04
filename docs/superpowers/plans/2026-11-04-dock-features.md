# Dock Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The dock gets Nectar's core behaviors: full right-click menu with pin/unpin, hover window previews with a count badge, drag-to-reorder pins, drive icons, Win+1..9, custom icons and an "Add app" panel.

**Architecture:** Windows stays the source of the pinned list (`pinned_apps.rs`, done). MacDows layers its own order (`dock.pinned_order`), custom icons (`dock.custom_icons`) and the pins Windows refused (`dock.local_pins`) on top; all of it lives in a nested `dock` settings object owned by Rust. Floating UI (previews, Add app) are extra windows built on demand with the `tray` window's pattern (`toggle_tray_panel` in `lib.rs`), selected by window label in `src/main.tsx`. Decision logic (ordering, menu contents, shortcut mapping, hover close) is pure functions with unit tests; Win32 calls stay thin.

**Tech Stack:** Rust (Tauri 2, `windows` 0.61, serde), React 19 + TypeScript, framer-motion, `cargo test`, `node --test src/**/*.test.ts` (Node 26 runs `.ts` tests directly).

**Spec:** `docs/superpowers/specs/2026-11-04-dock-features-design.md` (approved 2026-11-04; "Add app" included, calendar icon excluded).

## Global Constraints

- Settings keys and defaults exactly as spec §3: `dock.pinned_order: []`, `dock.local_pins: []`, `dock.custom_icons: {}`, `dock.show_drives: true`, `dock.previews: true`, `dock.win_number: true`.
- Windows' own pinned list is never reordered or edited except through the `taskbarpin` / `taskbarunpin` shell verbs (unpin was verified on this PC with HP Support Assistant; pin is untested).
- Custom icons are stored as 128×128 PNG in `%APPDATA%\MacDows\icons\`; the original file is never kept or changed.
- Nothing in the new code does background work when the dock is hidden/off (spec §5.6); the preview refresh timer runs only while a preview is open.
- Every new floating window must not steal focus (`focused(false)`), must be registered with `windows_list::register_bar` (so it never shows up as a dock icon), and closes when the dock hides.
- Commit style: lowercase conventional (`feat:`, `fix:`), author Fatal-IV, **no Claude/co-author lines**. All strings in `src/locales/en.json` and `tr.json`; native menu labels in `src-tauri/src/i18n.rs`.
- Real-dock verification after each task: build release (`bun run tauri build --no-bundle`, stop `macdows` first), run it, reveal the dock with the pointer at the bottom centre, capture only the bottom strip.

## Review Focus

- A saved `pinned_order` that mentions removed pins or misses newly pinned ones → removed ones dropped, new ones appended in Windows order. (Task 3)
- A pinned app with several windows, one minimized: preview lists all, click restores the minimized one. (Task 2)
- A window whose thumbnail capture fails or is black → its app icon is shown, no error spam. (Task 2)
- Win+N pressed while the pinned list is shorter than N, or while a game is fullscreen → key falls through to Windows / works without showing the dock. (Task 6)
- Custom-icon file picked is not an image or is huge → rejected with a visible message, original icon kept. (Task 4)
- A drive that disappears while its icon is under the pointer or its preview is open → entry vanishes without a crash. (Task 5)

---

## File Structure

- `src-tauri/src/settings.rs` — modify: `Dock` model (`dock` object).
- `src-tauri/src/pinned_apps.rs` — modify: local pins, order, custom icons applied in `list_pinned`; menu per target; pin/unpin/new window/quit.
- `src-tauri/src/preview.rs` — create: thumbnails, preview window lifecycle, hover debounce.
- `src-tauri/src/drives.rs` — create: drive list and watcher, `open_drive`.
- `src-tauri/src/hotkeys.rs` — create: Win+1..9 (`RegisterHotKey`, fallback hook).
- `src-tauri/src/custom_icons.rs` — create: save/remove/resolve icon files.
- `src-tauri/src/app_catalog.rs` — create: installed-app list for "Add app".
- `src/dock/Dock.tsx` (modify), `dockItems.ts` (modify), `dockDrag.ts` (create), `DockPreview.tsx` (create), `AddApp.tsx` (create), `src/main.tsx` (labels `preview`, `addapp`), `src/bridge.ts`, locales, `src/settings/Settings.tsx`.

---

### Task 1: Dock settings model

**Files:**
- Modify: `src-tauri/src/settings.rs`, `src/bridge.ts`
- Test: `src-tauri/src/settings.rs` (existing `mod tests`)

**Interfaces:**
- Produces (Rust, `#[serde(default)]`, derive `Debug, Clone, PartialEq, Serialize, Deserialize`): `Dock { pinned_order: Vec<String>, local_pins: Vec<LocalPin>, custom_icons: HashMap<String, String>, show_drives: bool, previews: bool, win_number: bool }`; `LocalPin { id: String, name: String, exe: String, aumid: String }`; `Settings.dock: Dock`.
- Produces (TS): `Dock`, `LocalPin` interfaces mirroring the above (`custom_icons: Record<string,string>`); `Settings.dock: Dock`.

- [ ] **Step 1: Failing tests** — `dock_defaults_match_the_spec` (all six defaults); `a_partial_dock_object_is_filled_with_defaults` (`{"dock":{"previews":false}}` keeps `show_drives == true`); `an_old_settings_file_without_dock_loads` (`{"language":"tr"}` → defaults).
- [ ] **Step 2: Run** `cargo test settings` — Expected: FAIL (compile error).
- [ ] **Step 3: Implement** the structs and the `Settings.dock` field; mirror in `bridge.ts`.
- [ ] **Step 4: Run** `cargo test` and `node node_modules/typescript/bin/tsc --noEmit` — Expected: all pass, tsc clean.
- [ ] **Step 5: Commit** `feat: dock settings model`.

### Task 2: Right-click menu, pin and unpin, new window, quit app

**Files:**
- Modify: `src-tauri/src/pinned_apps.rs`, `src-tauri/src/i18n.rs`, `src-tauri/src/lib.rs` (command registration), `src/bridge.ts`, `src/dock/Dock.tsx`, `src/dock/dockItems.ts`
- Test: `src-tauri/src/pinned_apps.rs`

**Interfaces:**
- Produces (Rust): `pub enum MenuTarget { Pinned, Running, Empty }`; `pub enum MenuAction { NewWindow, Unpin, Pin, ChangeIcon, ResetIcon, Quit, AddApp, MacdowsSettings, TaskManager }`; pure `pub fn menu_plan(target: &MenuTarget, running: bool, has_custom_icon: bool) -> Vec<MenuAction>` returning, in order: Pinned → `NewWindow, Unpin, ChangeIcon, [ResetIcon if has_custom_icon], [Quit if running]`; Running → `NewWindow, Pin, Quit`; Empty → `AddApp, MacdowsSettings, TaskManager`.
- Produces (commands): `dock_menu(app: AppHandle, target: DockMenuTarget) -> Result<(), String>` replacing `pinned_menu`, where `DockMenuTarget { kind: "pinned"|"running"|"empty", id: String, exe: String, aumid: String, name: String, hwnds: Vec<isize> }`; `launch_new(id_or_exe: String)`; `close_app_windows(hwnds: Vec<isize>)` (posts `WM_CLOSE` to each).
- Local pins: `list_pinned` appends `settings.dock.local_pins` (as `PinnedApp`, id prefix `local:`) after Windows' pins; `unpin` removes a `local:` pin from settings instead of calling Windows.
- `pin_app(app, exe, aumid, name)`: runs the `taskbarpin` verb (same `ShellExecuteExW` mechanism as `windows_unpin`) on the exe path or `shell:AppsFolder\<aumid>`; after up to 3 s, if the pin did not appear in `pins()`, adds a `LocalPin` and emits `pinned-changed`.

- [ ] **Step 1: Failing tests** for `menu_plan` (the three targets, `has_custom_icon` and `running` flags) and `local_pin_survives_when_windows_list_is_empty` (pure `merge_local(windows: Vec<PinnedApp>, local: &[LocalPin]) -> Vec<PinnedApp>` appends local pins whose ids are not already present).
- [ ] **Step 2: Run** `cargo test pinned_apps` — Expected: FAIL.
- [ ] **Step 3: Implement** the types, `menu_plan`, `merge_local`, the menu builder (one `MenuItem` per action, id `dock-menu:<action>:<payload-id>`, labels from `i18n::dock_menu_label(lang, action)` with Turkish: "Yeni pencere", "Görev çubuğundan kaldır", "Görev çubuğuna sabitle", "Simgeyi değiştir", "Simgeyi sıfırla", "Uygulamadan çık", "Uygulama ekle", "MacDows Ayarları", "Görev Yöneticisi"), and `on_menu_event` dispatch. `launch_new` starts the exe (`CreateProcess` via `std::process::Command` without a console window) or `ShellExecute open` for `.lnk`/AppsFolder ids.
- [ ] **Step 4: Frontend** — `Dock.tsx` right-click on any entry calls `dockMenu({kind, ...})`; right-click on empty dock area (the `.dock-stage` background) calls it with `kind:"empty"`. `DockEntry` gains `exe`, `aumid`, `hwnds`.
- [ ] **Step 5: Run** `cargo test`, `tsc --noEmit`; then real-dock check: right-click Edge → menu shows 4–5 items in Turkish; "Yeni pencere" opens a new Edge window; right-click Spotify (running, unpinned) → "Görev çubuğuna sabitle" pins it (report whether Windows accepted or MacDows kept it locally).
- [ ] **Step 6: Commit** `feat: dock right-click menu with pin, new window and quit`.

### Task 3: Drag to reorder pins (saved order)

**Files:**
- Create: `src/dock/dockDrag.ts`, `src/dock/dockDrag.test.ts`
- Modify: `src-tauri/src/pinned_apps.rs` (apply order in `list_pinned`), `src/dock/Dock.tsx`, `src/bridge.ts` (`setPinnedOrder(ids: string[])` → `set_setting("dock", …)` via a small command `set_pinned_order`)

**Interfaces:**
- Produces (Rust): `pub fn apply_order(pins: Vec<PinnedApp>, saved: &[String]) -> Vec<PinnedApp>` — saved ids (case-insensitive) first in saved order, unknown pins appended in input order, saved ids with no pin ignored.
- Produces (TS): `reorder(ids: string[], from: number, to: number): string[]` (pure, returns a new array, clamps `to`), `command set_pinned_order(app, ids: Vec<String>)` writing `dock.pinned_order`.

- [ ] **Step 1: Failing tests** — Rust: `apply_order_keeps_saved_order_and_appends_new_pins`, `apply_order_ignores_removed_pins`, `apply_order_is_case_insensitive`. TS: `reorder([a,b,c],0,2) → [b,c,a]`, `reorder([a,b,c],2,0) → [c,a,b]`, `reorder` with `to` out of range clamps, `from === to` returns equal copy.
- [ ] **Step 2: Run** both suites — Expected: FAIL.
- [ ] **Step 3: Implement** `apply_order`, `reorder`, `set_pinned_order`; call `apply_order` at the end of `list_pinned` (after `merge_local`).
- [ ] **Step 4: UI** — pinned `motion.button`s use framer-motion `Reorder.Group axis="x"` / `Reorder.Item` (already a dependency); on drag end call `setPinnedOrder(new ids)`. Esc during drag restores. Non-pinned entries and system buttons are not draggable.
- [ ] **Step 5: Verify** `cargo test`, `node --test src/dock/*.test.ts`, `tsc`; real dock: drag Edge before Explorer, restart MacDows — order kept; Windows' own list (`reg query ...Taskband`, or the Quick Launch folder) unchanged.
- [ ] **Step 6: Commit** `feat: reorder pinned dock apps by dragging`.

### Task 4: Custom icons

**Files:**
- Create: `src-tauri/src/custom_icons.rs`
- Modify: `src-tauri/src/pinned_apps.rs` (menu actions `ChangeIcon`/`ResetIcon`, icon override in `list_pinned`), `src-tauri/src/lib.rs`, `src/dock/Dock.tsx`, `src/bridge.ts`, locales
- Test: `src-tauri/src/custom_icons.rs`

**Interfaces:**
- Produces: `pub fn icon_file_name(id: &str) -> String` (stable, filesystem-safe: lowercase hex of a hash of the lowercased id + `.png`); `pub fn fit_to_square(img: image::DynamicImage, side: u32) -> image::DynamicImage` (letterboxed, transparent padding, keeps aspect); `pub fn save(id: &str, source: &Path) -> Result<String, String>` (decode with the `image` crate, reject files over 10 MB or that do not decode, write 128×128 PNG into the icons folder, return the file name); `pub fn remove(id: &str)`; `pub fn load_base64(file: &str) -> Option<String>`.
- Dock-visible: `PinnedApp.icon_png_base64` is replaced by the custom icon when `dock.custom_icons` has the id; running non-pinned entries get the override in the frontend via a new `custom_icons` map returned by `get_custom_icons() -> HashMap<String, String>` (id → base64).

- [ ] **Step 1: Failing tests** — `icon_file_name_is_stable_and_case_insensitive`, `fit_to_square_keeps_aspect_and_pads` (200×100 → 128×128 with transparent top/bottom rows), `save_rejects_a_file_that_is_not_an_image` (temp file with text), `save_writes_a_128_png` (generate a PNG in a temp dir).
- [ ] **Step 2: Run** `cargo test custom_icons` — Expected: FAIL.
- [ ] **Step 3: Implement** the module; the file picker uses `rfd`-free approach: a Tauri command `choose_icon_file()` calling `PowerShell`-free native `GetOpenFileNameW` (`Win32_UI_Controls_Dialogs` feature) with filter `Images (*.png;*.jpg;*.jpeg;*.ico;*.bmp)`; for `.exe` sources use the existing `icon_png_base64`.
- [ ] **Step 4: Wire** menu actions: `ChangeIcon` → picker → `save` → settings update (`custom_icons[id] = file`) → `pinned-changed`; failure shows an in-dock toast (a short-lived text above the dock via event `dock-toast`) with the reason. `ResetIcon` → `remove` + settings update.
- [ ] **Step 5: Verify** tests; real dock: change Edge's icon to a PNG, restart, still custom; reset restores; picking a `.txt` shows the message and keeps the icon.
- [ ] **Step 6: Commit** `feat: custom icons for dock apps`.

### Task 5: Drives

**Files:**
- Create: `src-tauri/src/drives.rs`
- Modify: `src-tauri/src/lib.rs`, `src/dock/Dock.tsx`, `src/bridge.ts`, `src/dock/dockItems.ts`, `src/settings/Settings.tsx`, locales, `src/dock/Dock.css`
- Test: `src-tauri/src/drives.rs`

**Interfaces:**
- Produces: `#[derive(Serialize)] pub struct Drive { letter: String, kind: String /* "local" | "removable" */, label: String }`; pure `pub fn kind_of(drive_type: u32) -> Option<&'static str>` (`DRIVE_FIXED(3)` → local, `DRIVE_REMOVABLE(2)` → removable, everything else `None`); `pub fn letters(mask: u32) -> Vec<char>` (bits of `GetLogicalDrives`); commands `list_drives() -> Vec<Drive>` (empty when `dock.show_drives` is false), `open_drive(letter: String)` (`ShellExecute open "<letter>:\"`, letter validated `A`–`Z`); watcher thread emitting `drives-changed` when the fingerprint (`GetLogicalDrives` + types) changes, polling every 3 s only while `show_drives` is true.

- [ ] **Step 1: Failing tests** — `kind_of` for 2, 3, 4 (network), 5 (CD), 6 (RAM); `letters(0b101)` → `['A','C']`; `open_drive` rejects `"1"`, `""`, `"CC"`.
- [ ] **Step 2: Run** `cargo test drives` — Expected: FAIL.
- [ ] **Step 3: Implement**; the label comes from `GetVolumeInformationW` (empty → "Local disk"/"Removable drive" via i18n).
- [ ] **Step 4: UI** — after the pinned block and a separator, one `dock-item` per drive (drive glyph + letter), click → `openDrive`; refresh on `drives-changed`; hidden when `show_drives` is false. Settings row "Show drives" (`settings.showDrives`).
- [ ] **Step 5: Verify** tests; real dock: C: appears and opens in Explorer; switch the setting off → gone.
- [ ] **Step 6: Commit** `feat: drives in the dock`.

### Task 6: Win+1..9

**Files:**
- Create: `src-tauri/src/hotkeys.rs`
- Modify: `src-tauri/src/lib.rs`, `src/dock/Dock.tsx`, `src/bridge.ts`, `src/settings/Settings.tsx`, locales
- Test: `src-tauri/src/hotkeys.rs`

**Interfaces:**
- Produces: pure `pub fn shortcut_for(vk: u32, shift: bool) -> Option<(usize, bool)>` (`'1'..='9'` → index 0..8, `bool` = new instance when Shift held); `pub fn start(app: AppHandle)`; command `set_dock_order(ids: Vec<String>)` — the dock tells Rust the current pinned ids in display order after every change (stored in a `Mutex<Vec<String>>`); event `dock-shortcut` with payload `{ index: usize, new_instance: bool }` sent to the `dock` window (the dock webview then focuses the first window or launches the pin, or launches a new instance).
- Registration order: `RegisterHotKey(None, id, MOD_WIN | MOD_NOREPEAT, vk)` for 1–9 and `MOD_WIN | MOD_SHIFT` ones, on a dedicated message-loop thread. If any registration fails, drop all and install a `WH_KEYBOARD_LL` hook on the same thread that swallows only `VK_1..VK_9` while a Win key is down and `index < pinned count`, forwarding everything else untouched; the hook procedure only posts to a channel (no work in the hook). The fallback emits one event `hotkey-fallback` so the dock shows a toast once per run.
- Off switch: `dock.win_number == false` stops the thread and unregisters/unhooks.

- [ ] **Step 1: Failing tests** — `shortcut_for('1', false) == Some((0,false))`, `('9', true) == Some((8,true))`, `('0', false) == None`, `('A', false) == None`; `should_swallow(index: usize, pinned_count: usize) -> bool` false when `index >= pinned_count`.
- [ ] **Step 2: Run** `cargo test hotkeys` — Expected: FAIL.
- [ ] **Step 3: Implement** the thread, registration with fallback, `set_dock_order`, event emit; settings change starts/stops it (listen to the same `settings-changed` path in `lib.rs` `set_setting` match: add `"dock" => hotkeys::apply(&app, updated.dock.win_number)`).
- [ ] **Step 4: UI** — `Dock.tsx` listens to `dock-shortcut`; for `index` within pinned entries: running → focus (cycle windows on repeated press), else launch; `new_instance` → `launchNew`. After every pinned-list change it calls `setDockOrder(ids)`.
- [ ] **Step 5: Verify** tests; real run: report whether `RegisterHotKey` succeeded or the hook fallback engaged (log line `[hotkeys] mode=...`), then press Win+1 via `keybd_event` and confirm the first pin comes forward/starts, and that Win+9 with fewer pins reaches Windows (opens nothing harmful).
- [ ] **Step 6: Commit** `feat: Win+1..9 start or focus the pinned apps`.

### Task 7: Window previews and count badge

**Files:**
- Create: `src-tauri/src/preview.rs`, `src/dock/DockPreview.tsx`, `src/dock/Preview.css`
- Modify: `src-tauri/src/lib.rs`, `src/main.tsx` (label `preview`), `src/dock/Dock.tsx`, `src/dock/Dock.css` (badge), `src/bridge.ts`, `src/settings/Settings.tsx`, locales
- Test: `src-tauri/src/preview.rs`

**Interfaces:**
- Produces (Rust): `pub fn fit_size(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32)` (keeps aspect, never upscales); `pub fn thumbnail(hwnd: isize, max_w: u32, max_h: u32) -> Option<String>` (`PrintWindow(PW_RENDERFULLCONTENT)` into a memory DC, scale, base64 PNG via `windows_list::hbitmap_to_png_base64`-style encoding; `None` when the bitmap is entirely black or the window is gone); commands `show_preview(app, center_x: f64, dock_top: f64, title: String, hwnds: Vec<isize>)`, `preview_hover(inside: bool)`, `close_preview(app)`, `preview_items() -> Vec<PreviewItem>` where `PreviewItem { hwnd: isize, title: String, thumbnail: Option<String> }`, `preview_activate(hwnd: isize)`, `preview_close_window(hwnd: isize)` (posts `WM_CLOSE`, then drops the item).
- Pure close logic: `pub fn should_close(dock_hover: bool, preview_hover: bool, since_left_ms: u64) -> bool` = both false and `since_left_ms >= 250`.
- Window: label `preview`, built like `tray` (no decorations, transparent, always on top, skip taskbar, `focused(false)`, registered with `register_bar`), positioned centred above the hovered icon (`center_x`, `dock_top` as in `toggle_tray_panel`), size from item count (card 200×140 + gaps, max 5 per row).
- Refresh: while open, a timer thread every 1 s re-captures thumbnails and emits `preview-updated`; it stops when the window closes.

- [ ] **Step 1: Failing tests** — `fit_size(1920,1080,200,120) == (200,112)`; `fit_size(100,50,200,120) == (100,50)` (no upscale); `fit_size(0,0,200,120)` returns `(0,0)` without dividing by zero; `should_close` table (both hover false and 250 ms → true; either hover true → false; 100 ms → false).
- [ ] **Step 2: Run** `cargo test preview` — Expected: FAIL.
- [ ] **Step 3: Implement** the module and the React view (`DockPreview`: cards with title, thumbnail or the app icon fallback, a close button; click → `previewActivate`, which restores a minimized window with `ShowWindow(SW_RESTORE)` then `SetForegroundWindow`; Esc closes). `Dock.tsx`: on pointer-enter of an entry with ≥1 window and `dock.previews` on, after 300 ms call `showPreview` with the entry's hwnds and the button's centre; pointer-leave/enter report `preview_hover`; entries with ≥2 windows show a count badge (`.dock-badge`). The preview closes when the dock hides (`visibility.rs` hide path calls `preview::close`).
- [ ] **Step 4: Verify** tests; real dock: hover Edge → preview with one card per Edge window; click a card focuses it; close button closes that window; Spotify (GPU-drawn) shows a thumbnail or the icon; hover away closes after ~250 ms; setting off → no preview.
- [ ] **Step 5: Commit** `feat: dock window previews with count badge`.

### Task 8: Add app panel

**Files:**
- Create: `src-tauri/src/app_catalog.rs`, `src/dock/AddApp.tsx`, `src/dock/AddApp.css`
- Modify: `src-tauri/src/lib.rs`, `src-tauri/src/pinned_apps.rs` (`MenuAction::AddApp`), `src/main.tsx` (label `addapp`), `src/bridge.ts`, locales
- Test: `src-tauri/src/app_catalog.rs`

**Interfaces:**
- Produces: `#[derive(Serialize, Clone)] pub struct CatalogApp { id: String, name: String, exe: String, aumid: String, icon_png_base64: Option<String>, running: bool, pinned: bool }`; pure `pub fn filter_apps(apps: &[CatalogApp], query: &str) -> Vec<CatalogApp>` (case-insensitive substring on `name`, empty query returns all, sorted: running first then alphabetical by lowercase name); `pub fn read_catalog() -> Vec<CatalogApp>` (Start menu shortcuts from `%ProgramData%` and `%APPDATA%\Microsoft\Windows\Start Menu\Programs`, resolved via the same `shortcut_target`; Store apps by enumerating `shell:AppsFolder` through `IShellItem` children; duplicates by lowercased exe/aumid removed; `.lnk` named Uninstall/Readme skipped); commands `open_add_app_panel(app, center_x, dock_top)`, `get_catalog() -> Vec<CatalogApp>`, `pin_catalog_app(app, id)` (uses `pin_app` from Task 2), `close_add_app_panel(app)`.
- Window: label `addapp`, same pattern as `tray`, 360×440, takes focus (search box), closes on blur/Esc.

- [ ] **Step 1: Failing tests** — `filter_apps` (empty query, case-insensitive match, running-first ordering, no match → empty); `read_catalog_drops_duplicates` using a pure `dedupe(apps: Vec<CatalogApp>) -> Vec<CatalogApp>`.
- [ ] **Step 2: Run** `cargo test app_catalog` — Expected: FAIL.
- [ ] **Step 3: Implement** catalog reading (off the UI thread, cached for 30 s), the panel view (search box, sections "Running" and "All apps", pinned ones greyed with a check), click → `pin_catalog_app`, panel stays open for further picks.
- [ ] **Step 4: Verify** tests; real dock: right-click empty dock → "Uygulama ekle", search "calc", click → Calculator pins (Windows or locally, report which) and appears in the dock; already-pinned apps are greyed.
- [ ] **Step 5: Commit** `feat: add apps to the dock from a searchable list`.

### Task 9: Settings switches, texts, changelog

**Files:**
- Modify: `src/settings/Settings.tsx`, `src/locales/en.json`, `src/locales/tr.json`, `CHANGELOG.md`

- [ ] **Step 1:** Bars page rows for `show_drives`, `previews`, `win_number` (toggles writing `dock.<key>` through the whole-`dock` object so other fields survive: helper `updateDock(s, patch)`), plus a "Hidden pinned apps" reset kept from the pinned-apps work.
- [ ] **Step 2:** Locale completeness check: `node -e` comparing the key sets of `en.json` and `tr.json` prints no differences.
- [ ] **Step 3:** CHANGELOG entry under Unreleased listing all dock features.
- [ ] **Step 4: Final verification:** `cargo test`, `node --test src/**/*.test.ts`, `node node_modules/typescript/bin/tsc --noEmit`, `bun run tauri build --no-bundle`; run the real app once through every feature with an old `settings.json` copied aside first to confirm it still loads.
- [ ] **Step 5: Commit** `feat: dock settings switches and texts`.

---

## Self-Review

- **Spec coverage:** §3 model → Task 1; §4.1 menu/pin/new window/quit → Task 2; §4.2 preview + badge → Task 7; §4.3 drag → Task 3; §4.4 drives → Task 5; §4.5 Win+N (RegisterHotKey then hook, bildirim) → Task 6; §4.6 custom icon → Task 4; §4.7 Add app → Task 8; §4.8 settings → Task 9 (+ per-task rows); §5 rules → Global Constraints and Review Focus; §6 phases map to Tasks 2–8 (phase order in the spec is menu, preview, order+icon, drives, hotkey, add app; this plan builds order/icon before previews because previews depend on the menu and window counts but not on order — preview moved to Task 7 so the cheaper, lower-risk features land first; no feature dropped).
- **Type consistency:** `DockMenuTarget`/`MenuAction` defined in Task 2 and used in Tasks 4 and 8; `apply_order` (Task 3) runs after `merge_local` (Task 2); `custom_icons` override applied in `list_pinned` (Task 4) after `apply_order`; `set_dock_order` (Task 6) receives ids after all three; `pin_app` (Task 2) is reused by Task 8.
- **Open for the user:** the order of Tasks 3–7 differs from the spec's phase order (previews last). Say if previews should come earlier.
