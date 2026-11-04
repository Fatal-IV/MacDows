# Graph Report - MacDows  (2026-11-04)

## Corpus Check
- 127 files · ~103,369 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .css 11, (none) 6, .ico 3)

## Summary
- 2021 nodes · 3610 edges · 115 communities (82 shown, 33 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 94 edges (avg confidence: 0.92)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `73e25f20`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- windows_list.rs
- shelf.rs
- DynamicIsland.tsx
- media.rs
- package.json
- tauri.conf.json
- settings.rs
- compilerOptions
- src-tauri/src/lib.rs
- compilerOptions
- Dock.tsx
- default.json
- islandLogic.test.ts
- macdows
- system_theme.rs
- bluetooth.rs
- updates.rs
- visualizer.rs
- api.ts
- tray_panel.rs
- bridge.ts
- install.rs
- react
- theme.ts
- taskbar_plan.rs
- installer/package.json
- dock_menu.rs
- logic.rs
- island_settings.rs
- hide
- Window
- compilerOptions
- taskbar.rs
- installer/src-tauri/tauri.conf.json
- MacDows Installer and Logo — Design
- make-icons.mjs
- quick_settings.rs
- rig
- Dynamic Island Settings Implementation Plan
- MacDows Taskbar Takeover — Design
- launcher.rs
- Handoff — where MacDows stands (2026-11-02)
- Dinamik Ada ayarları: tasarım
- run_install
- build-installer.mjs
- pinned_apps.rs
- fake-update-server.mjs
- Installer facts verified on a real PC (2026-11-02)
- In-app update facts (2026-11-02)
- MacDows In-App Update — Design
- SetupError
- hotkeys.rs
- installer/src-tauri/capabilities/default.json
- notifications.rs
- Taskbar takeover facts verified on a real PC (2026-11-02, Windows 11, one monitor)
- Stage
- 4. Özellikler
- MacDows
- MacDows.Setup.csproj
- useT
- visibility.rs
- macdows-setup
- QuickSettings.tsx
- custom_icons.rs
- File Structure
- File Structure
- collect
- Settings.tsx
- drives.rs
- i18n.rs
- preview.rs
- secure_store.rs
- TaskbarState
- super
- Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım
- Backend
- ShellAction
- File Structure
- app_catalog.rs
- devDependencies
- extract-notes.mjs
- scripts
- dependencies
- window_icon_png_base64
- OnceLock
- LPARAM
- AtomicU32

## God Nodes (most connected - your core abstractions)
1. `Notch()` - 41 edges
2. `useT()` - 30 edges
3. `Dock()` - 28 edges
4. `TaskbarState` - 25 edges
5. `react` - 24 edges
6. `Window` - 18 edges
7. `UpdateStatus` - 17 edges
8. `initTheme()` - 16 edges
9. `QuickSettingsScreen()` - 16 edges
10. `compilerOptions` - 16 edges

## Surprising Connections (you probably didn't know these)
- `Task 8: Notification sub-features (phase 3, Rust + UI)` --references--> `NotifCfg`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts
- `Task 9: Shelf sub-features (phase 3, Rust + UI)` --references--> `ShelfCfg`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts
- `Task 1: Dock settings model` --references--> `Dock()`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dock-features.md → src/dock/Dock.tsx
- `Task 6: UI and uninstall hook` --references--> `useSystemStatus()`  [INFERRED]
  docs/superpowers/plans/2026-11-02-taskbar-takeover.md → src/system.ts
- `State` --references--> `main()`  [INFERRED]
  docs/HANDOFF.md → scripts/build-installer.mjs

## Import Cycles
- None detected.

## Communities (115 total, 33 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.08
Nodes (18): Store apps use window icon (WhatsApp fix), AppWindow, BARS, closed_windows_are_forgotten_and_reopened_go_to_the_end(), EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED, ids(), list_windows() (+10 more)

### Community 1 - "shelf.rs"
Cohesion: 0.12
Nodes (22): add_to_shelf(), clear_shelf(), CREATE_NO_WINDOW, current_session(), file_name(), get_shelf(), icon_for(), ICONS (+14 more)

### Community 2 - "DynamicIsland.tsx"
Cohesion: 0.09
Nodes (37): Task 6: Status, Music and Tools sub-features (phase 3), Props, silhouette(), SurfaceEdge, SurfaceShape(), addToShelf(), clearShelf(), getMedia() (+29 more)

### Community 3 - "media.rs"
Cohesion: 0.06
Nodes (36): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard), APP, attach_current() (+28 more)

### Community 4 - "package.json"
Cohesion: 0.12
Nodes (14): license, name, private, type, version, react-dom, @resvg/resvg-js, @tauri-apps/cli (+6 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.07
Nodes (29): Tag-based signed release (GitHub Actions), app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+21 more)

### Community 6 - "settings.rs"
Cohesion: 0.11
Nodes (27): settings.json (corrupt file falls back to defaults), a_bad_island_value_does_not_reset_the_other_settings(), a_local_pin_round_trips(), a_partial_dock_object_is_filled_with_defaults(), a_v010_settings_file_without_update_fields_still_loads(), an_existing_island_object_wins_over_the_old_key(), an_old_file_without_island_gets_every_island_default(), an_old_settings_file_without_dock_loads() (+19 more)

### Community 7 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 8 - "src-tauri/src/lib.rs"
Cohesion: 0.14
Nodes (23): Single instance: second launch opens Settings, activate_tray_item(), apply_replace_taskbar(), AppSettings, bring_to_front(), build_tray(), close_tray(), close_tray_panel() (+15 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "Dock.tsx"
Cohesion: 0.09
Nodes (34): Task 2: Right-click menu, pin and unpin, new window, quit app, AppWindow, dockMenu(), focusWindow(), getCustomIcons(), launchNewApp(), launchPinned(), listDrives() (+26 more)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "islandLogic.test.ts"
Cohesion: 0.23
Nodes (16): Task 2: Pure navigation logic (phase 2 core, TypeScript), Task 5: The island obeys screen and navigation settings (phase 2, runtime), ClickMode, FullscreenMode, Island, ScreenId, DEFAULT_ORDER, defaultIsland() (+8 more)

### Community 17 - "system_theme.rs"
Cohesion: 0.08
Nodes (29): accent_from_registry(), get_system_accent_color(), abgr(), ACCENT, apply(), Backup, backup_path(), broadcast() (+21 more)

### Community 18 - "bluetooth.rs"
Cohesion: 0.09
Nodes (14): address_hex(), batteries(), BATTERY_KEY, BATTERY_TIME_KEY, BtDevice, freshest(), get_bluetooth_devices(), kind_of() (+6 more)

### Community 19 - "updates.rs"
Cohesion: 0.07
Nodes (38): after_check(), available(), begin_check(), begin_install(), build_updater(), can_start(), check_for_update(), CHECK_INTERVAL_SECS (+30 more)

### Community 20 - "visualizer.rs"
Cohesion: 0.07
Nodes (31): a_louder_tone_shows_a_higher_bar(), a_steady_tone_lights_its_own_band(), analyze(), BANDS, bars_rise_at_once_and_fall_gradually(), capture_session(), CEIL_DB, EDGES (+23 more)

### Community 21 - "api.ts"
Cohesion: 0.08
Nodes (36): File Structure, Global Constraints, MacDows Installer and Logo Implementation Plan, Review Focus, Self-review, Task 1: Verify the real Windows facts before coding against them, Task 2: Logo sources and icon generation, Task 3: Pure installer logic (Rust, tests first) (+28 more)

### Community 22 - "tray_panel.rs"
Cohesion: 0.13
Nodes (25): activate(), automation(), BUSY, capture(), class_name(), close_flyout(), collect(), find_by_class() (+17 more)

### Community 23 - "bridge.ts"
Cohesion: 0.08
Nodes (26): Self-Review, closePreview(), Dock, DockMenuTarget, Drive, LocalPin, MediaInfo, mediaNext() (+18 more)

### Community 24 - "install.rs"
Cohesion: 0.08
Nodes (22): BUNDLED_VERSION, CREATE_NO_WINDOW, ErrorEvent, extract_payload(), extract_payload_works_with_non_ascii_dir(), install_state(), InstalledInfo, InstallInfo (+14 more)

### Community 25 - "react"
Cohesion: 0.11
Nodes (23): Task 4: UI, framer-motion, react, @tauri-apps/api, initTheme(), CatalogApp, closeAddAppPanel(), getCatalog() (+15 more)

### Community 26 - "theme.ts"
Cohesion: 0.13
Nodes (24): applyBackdrop(), clearBackdrop(), enableBackdrop(), STYLE_BLUR, usesBackdrop(), VARS, WallpaperBackdrop, wallpaperRect() (+16 more)

### Community 27 - "taskbar_plan.rs"
Cohesion: 0.09
Nodes (12): a_restart_after_a_crash_keeps_the_stored_original(), hidden(), hidden_icons_need_the_chevron_and_a_hidden_taskbar(), HidePlan, hiding_records_the_real_original(), plan_hide(), plan_restore(), restore_puts_back_each_original() (+4 more)

### Community 28 - "installer/package.json"
Cohesion: 0.07
Nodes (28): dependencies, react, react-dom, @tauri-apps/api, devDependencies, @tauri-apps/cli, @types/react, @types/react-dom (+20 more)

### Community 29 - "dock_menu.rs"
Cohesion: 0.07
Nodes (25): ALL_ACTIONS, change_icon(), CURRENT, dock_menu(), DockMenuTarget, icon_key(), menu_plan(), MenuAction (+17 more)

### Community 30 - "logic.rs"
Cohesion: 0.11
Nodes (13): decide(), Decision, Fresh, Newer, Older, Reinstall, exit_code_message(), exit_code_zero_is_a_caller_bug() (+5 more)

### Community 31 - "island_settings.rs"
Cohesion: 0.09
Nodes (26): a_broken_order_is_repaired_to_every_screen_once(), click_and_fullscreen_modes_use_lowercase_names(), ClickMode, Player, Quick, Smart, defaults_match_the_spec(), FullscreenMode (+18 more)

### Community 32 - "hide"
Cohesion: 0.16
Nodes (12): hide(), hide_then_restore_returns_to_auto_hide_off(), hide_then_restore_returns_to_auto_hide_on_when_it_was_on(), HideError, StateNotSaved, hiding_again_after_a_crash_keeps_the_first_original(), MemStore, MovesTheWindowAway (+4 more)

### Community 33 - "Window"
Cohesion: 0.07
Nodes (24): MacDows.Setup, Application, App, Bar, Box, DonePanel, DoneText, DoneTitle (+16 more)

### Community 34 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+9 more)

### Community 35 - "taskbar.rs"
Cohesion: 0.07
Nodes (32): ABS_AUTOHIDE, appbar_state(), battery_percent_from(), chevron_present_uia(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, current_status(), DETACHED_PROCESS (+24 more)

### Community 36 - "installer/src-tauri/tauri.conf.json"
Cohesion: 0.12
Nodes (16): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+8 more)

### Community 37 - "MacDows Installer and Logo — Design"
Cohesion: 0.13
Nodes (14): Architecture, Build and release, Decisions (user), Errors, Goal, Install behavior, Logo, MacDows Installer and Logo — Design (+6 more)

### Community 38 - "make-icons.mjs"
Cohesion: 0.14
Nodes (12): png-to-ico, appIcons, assets, icoSizes, installerIcons, main, pngFor(), render() (+4 more)

### Community 39 - "quick_settings.rs"
Cohesion: 0.07
Nodes (28): airplane_on(), airplane_tile(), banners_on(), clamp_percent(), CREATE_NO_WINDOW, get_brightness(), get_quick_settings(), has_battery() (+20 more)

### Community 40 - "rig"
Cohesion: 0.20
Nodes (4): FakeBackend, FakeStore, nothing_is_hidden_when_the_state_cannot_be_saved(), rig()

### Community 41 - "Dynamic Island Settings Implementation Plan"
Cohesion: 0.33
Nodes (5): Dynamic Island Settings Implementation Plan, Global Constraints, Review Focus, Status (2026-11-04, evening), Where the spec is silent (interpretations; no feature of the spec is dropped)

### Community 42 - "MacDows Taskbar Takeover — Design"
Cohesion: 0.15
Nodes (12): Behavior details, Decisions (user), Errors, Goal, How the taskbar is hidden, MacDows Taskbar Takeover — Design, Risks, Scope (+4 more)

### Community 43 - "launcher.rs"
Cohesion: 0.11
Nodes (18): BOOTSTRAPPER_URL, CLIENT_KEY, CREATE_NO_WINDOW, download(), download_args(), download_command_follows_redirects_and_names_the_target(), ensure_webview2(), first_pv() (+10 more)

### Community 44 - "Handoff — where MacDows stands (2026-11-02)"
Cohesion: 0.20
Nodes (9): Decisions to keep, Handoff — where MacDows stands (2026-11-02), Knowledge graph, Known gaps (deferred), Release secrets (important), Roadmap, Run it, Start and Search (2026-11-03) (+1 more)

### Community 45 - "Dinamik Ada ayarları: tasarım"
Cohesion: 0.17
Nodes (11): 1. Amaç, 2.1 Ekranlar, 2.2 Gezinme, 2.3 Görünürlük ve yerleşim, 2. Ayar modeli, 3. Davranış kuralları, 4. Arayüz, 5. Aşamalar (+3 more)

### Community 46 - "run_install"
Cohesion: 0.35
Nodes (9): apply_autostart_choice(), emit_stage(), fail(), launch_app(), run_install(), start_install(), system_locale(), write_log() (+1 more)

### Community 47 - "build-installer.mjs"
Cohesion: 0.16
Nodes (6): findPayload(), main(), root, run(), sumsLine(), root

### Community 48 - "pinned_apps.rs"
Cohesion: 0.07
Nodes (53): a_local_pin_windows_now_has_is_not_listed_twice(), a_store_app_named_twice_counts_once_at_its_first_mention(), app(), apply_order(), apply_order_ignores_case_and_leaves_an_empty_order_alone(), apply_order_ignores_pins_that_are_gone(), apply_order_keeps_the_saved_order_and_appends_new_pins(), APPS_FOLDER (+45 more)

### Community 49 - "fake-update-server.mjs"
Cohesion: 0.25
Nodes (6): [dir, version, portArg], manifest, pkg, port, signature, tamper

### Community 50 - "Installer facts verified on a real PC (2026-11-02)"
Cohesion: 0.29
Nodes (6): End-to-end run of the built installer (2026-11-02, this PC, Windows 11, Turkish UI), Installer facts verified on a real PC (2026-11-02), MacDows NSIS package (`MacDows_0.1.0_x64-setup.exe`, per-user), Not verified, Not verified (cannot be done here), WebView2

### Community 51 - "In-app update facts (2026-11-02)"
Cohesion: 0.29
Nodes (6): End-to-end run on this PC (2026-11-02), Fake release for the local test, From the plugin source (`tauri-plugin-updater` 2.14.1, `config.rs`, `updater.rs`), In-app update facts (2026-11-02), Not verified, Still to be answered by the end-to-end run (Task 5)

### Community 52 - "MacDows In-App Update — Design"
Cohesion: 0.09
Nodes (22): File Structure, Global Constraints, MacDows In-App Update Implementation Plan, Review Focus, Self-review, Task 1: Pure update logic and settings fields (tests first), Task 2: Prove the update path with a local fake release, Task 3: Update effects (state, background task, commands) (+14 more)

### Community 53 - "SetupError"
Cohesion: 0.40
Nodes (6): io(), run_nsis(), SetupError, Io, NotInstalled, PayloadMissing

### Community 54 - "hotkeys.rs"
Cohesion: 0.09
Nodes (16): auto_repeat_is_swallowed_without_firing_again(), decide(), Decision, HELD, hook_proc(), MASK_VK, OWN_TAG, PINNED_COUNT (+8 more)

### Community 55 - "installer/src-tauri/capabilities/default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 56 - "notifications.rs"
Cohesion: 0.09
Nodes (25): APP, BANNER_HOLD_MS, clear_notification_history(), fresh(), get_notification_history(), get_notifications(), HISTORY, history_is_capped() (+17 more)

### Community 57 - "Taskbar takeover facts verified on a real PC (2026-11-02, Windows 11, one monitor)"
Cohesion: 0.40
Nodes (4): Flyouts while the taskbar is hidden (screenshots checked), Hiding, Not verified, Taskbar takeover facts verified on a real PC (2026-11-02, Windows 11, one monitor)

### Community 58 - "Stage"
Cohesion: 0.40
Nodes (5): Stage, Done, Finishing, Installing, Preparing

### Community 59 - "4. Özellikler"
Cohesion: 0.11
Nodes (17): 1. Amaç, 2. Nectar'dan fark (bilerek), 3. Model, 4.1 Sağ tık menüsü, 4.2 Pencere önizlemesi ve birden çok pencere, 4.3 Sürükleyerek sıralama, 4.4 Sürücüler, 4.5 Win+1..9 (+9 more)

### Community 60 - "MacDows"
Cohesion: 0.40
Nodes (4): Develop, Install, MacDows, Updates

### Community 62 - "useT"
Cohesion: 0.10
Nodes (28): @crabnebula/tauri-plugin-drag, clearNotificationHistory(), getNotificationHistory(), getSettings(), Notice, onNotificationHistoryChanged(), onSettingsChanged(), openShelfItem() (+20 more)

### Community 66 - "visibility.rs"
Cohesion: 0.05
Nodes (33): Event-driven, no polling for windows and media, Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, APP, class_of(), covers_monitor(), LAST, MON (+25 more)

### Community 69 - "QuickSettings.tsx"
Cohesion: 0.08
Nodes (34): Task 7: Quick settings sub-features (phase 3), BtDevice, BtKind, getBluetoothDevices(), getBrightness(), getQuickSettings(), getVolume(), openMacdowsSettings() (+26 more)

### Community 70 - "custom_icons.rs"
Cohesion: 0.13
Nodes (17): a_file_that_is_not_an_image_is_rejected(), a_missing_file_is_rejected(), a_picture_is_saved_as_a_128_png(), a_wide_image_is_fitted_with_transparent_bars_above_and_below(), fit_to_square(), get_custom_icons(), icon_file_name(), icons_dir() (+9 more)

### Community 71 - "File Structure"
Cohesion: 0.14
Nodes (13): File Structure, Global Constraints, MacDows Taskbar Takeover Implementation Plan, Review Focus, Self-review, Task 1: Pure plans and key steps (tests first), Task 2: Hide and restore over traits (tests first), Task 3: Real Windows backend, state file and shell actions (+5 more)

### Community 72 - "File Structure"
Cohesion: 0.14
Nodes (13): Dock Features Implementation Plan, File Structure, Global Constraints, Review Focus, Self-Review, Task 1: Dock settings model, Task 3: Drag to reorder pins (saved order), Task 4: Custom icons (+5 more)

### Community 73 - "collect"
Cohesion: 0.21
Nodes (11): aumid_of(), class_of(), collect(), collect_rects(), exe_of(), focus_window(), is_bar_window(), is_listable() (+3 more)

### Community 74 - "Settings.tsx"
Cohesion: 0.11
Nodes (37): File Structure, Task 10: Reset buttons, texts, polish (phase 4), Task 1: Island settings model, migration, normalization (phase 1, Rust), Task 3: Settings page skeleton, visibility/scale/fullscreen moved (phase 1, UI), Task 4: Screens and navigation sections in the settings page (phase 2, UI), Task 8: Notification sub-features (phase 3, Rust + UI), Task 9: Shelf sub-features (phase 3, Rust + UI), setSetting() (+29 more)

### Community 75 - "drives.rs"
Cohesion: 0.14
Nodes (12): Drive, DRIVE_FIXED, DRIVE_REMOVABLE, enabled(), kind_of(), letters(), list_drives(), open_drive() (+4 more)

### Community 76 - "i18n.rs"
Cohesion: 0.18
Nodes (3): Languages: English and Turkish, taskbar_notice(), the_first_run_notice_explains_how_to_undo_it()

### Community 77 - "preview.rs"
Cohesion: 0.06
Nodes (35): WIN_DOWN, CARD_H, CARD_W, CLOSE_DELAY_MS, close_panel(), close_preview(), DOCK_HOVER, fit_size() (+27 more)

### Community 78 - "secure_store.rs"
Cohesion: 0.26
Nodes (8): blob(), MAGIC, open(), plain_files_and_damaged_data_are_not_opened(), seal(), take(), the_sealed_bytes_do_not_contain_the_text(), what_is_sealed_can_be_opened_again()

### Community 79 - "TaskbarState"
Cohesion: 0.14
Nodes (9): guard_should_restore(), needs_final_show(), needs_stale_restore(), state_belongs_to(), state_from_json(), state_to_json(), Store, TaskbarState (+1 more)

### Community 80 - "super"
Cohesion: 0.25
Nodes (5): Cli, Guard, Normal, RestoreTaskbar, parse()

### Community 81 - "Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım"
Cohesion: 0.15
Nodes (12): 1. Amaç, 2.1 Wi-Fi sayfası, 2.2 Şifre ve klavye odağı, 2.3 Bluetooth sayfası, 2.4 VPN sayfası, 2. Arayüz, 3. Ayarlar, 4. Teknik tasarım (+4 more)

### Community 83 - "ShellAction"
Cohesion: 0.17
Nodes (12): hidden_icons_jump_to_the_chevron_with_home_before_pressing_enter(), key_steps(), KeyStep, Chord, Wait, ShellAction, HiddenIcons, Notifications (+4 more)

### Community 84 - "File Structure"
Cohesion: 0.17
Nodes (11): Control Center Network Pages Implementation Plan, File Structure, Global Constraints, Review Focus, Self-Review, Task 1: Wi-Fi backend: adapters, scan, list, profile XML (no connecting yet), Task 2: Wi-Fi page: list, connect to saved or open networks, disconnect, forget, Task 3: Password entry, new secured networks, hidden networks, keyboard focus (+3 more)

### Community 93 - "app_catalog.rs"
Cohesion: 0.10
Nodes (27): app(), CACHE, CACHE_FOR, CatalogApp, close_add_app_panel(), collect_shortcuts(), dedupe(), duplicates_by_program_or_app_id_keep_the_first() (+19 more)

### Community 94 - "devDependencies"
Cohesion: 0.22
Nodes (9): devDependencies, png-to-ico, @resvg/resvg-js, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite (+1 more)

### Community 95 - "extract-notes.mjs"
Cohesion: 0.50
Nodes (3): end, lines, start

### Community 96 - "scripts"
Cohesion: 0.25
Nodes (8): scripts, build, dev, icons, preview, tauri, test:logic, test:scripts

### Community 97 - "dependencies"
Cohesion: 0.29
Nodes (7): dependencies, @crabnebula/tauri-plugin-drag, framer-motion, react, react-dom, @tauri-apps/api, @tauri-apps/plugin-opener

### Community 99 - "window_icon_png_base64"
Cohesion: 0.33
Nodes (5): hbitmap_to_png_base64(), hicon_to_png_base64(), icon_png_base64(), rgba_to_png_base64(), window_icon_png_base64()

### Community 101 - "OnceLock"
Cohesion: 0.29
Nodes (4): APP, MANAGER, APP, start()

### Community 102 - "LPARAM"
Cohesion: 0.40
Nodes (3): apply(), enumerate(), visible_rects()

## Knowledge Gaps
- **495 isolated node(s):** `Status (2026-11-04, evening)`, `Global Constraints`, `Where the spec is silent (interpretations; no feature of the spec is dropped)`, `Review Focus`, `Task 1: Island settings model, migration, normalization (phase 1, Rust)` (+490 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 853 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Dock (running apps, stable order)` connect `media.rs` to `windows_list.rs`, `Dock.tsx`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Dock()` (e.g. with `File Structure` and `Task 1: Dock settings model`) actually correct?**
  _`Dock()` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Status (2026-11-04, evening)`, `Global Constraints`, `Where the spec is silent (interpretations; no feature of the spec is dropped)` to the rest of the system?**
  _495 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.08064516129032258 - nodes in this community are weakly interconnected._
- **Why does `Visibility modes Fixed/Smart/Peek` connect `visibility.rs` to `Settings.tsx`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Should `shelf.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12096774193548387 - nodes in this community are weakly interconnected._
- **Why does `@tauri-apps/api` connect `react` to `DynamicIsland.tsx`, `package.json`, `Dock.tsx`, `api.ts`, `bridge.ts`, `theme.ts`?**
  _High betweenness centrality (0.044) - this node is a cross-community bridge._