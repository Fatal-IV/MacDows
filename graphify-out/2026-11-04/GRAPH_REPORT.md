# Graph Report - MacDows  (2026-11-04)

## Corpus Check
- 136 files · ~111,994 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 26 file(s) not represented in the graph (top: .css 11, (none) 6, .ico 3)

## Summary
- 2203 nodes · 4015 edges · 124 communities (101 shown, 23 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 100 edges (avg confidence: 0.92)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d9a1000c`
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
- dock_menu.rs
- compilerOptions
- bridge.ts
- default.json
- bluetooth.rs
- macdows
- system_theme.rs
- wifi.rs
- updates.rs
- visualizer.rs
- api.ts
- tray_panel.rs
- Dock.tsx
- install.rs
- Settings.tsx
- theme.ts
- taskbar_plan.rs
- installer/package.json
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
- bluetooth_pairing.rs
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
- fullscreen.rs
- macdows-setup
- QuickSettings.tsx
- custom_icons.rs
- File Structure
- File Structure
- MenuAction
- IslandSettings.tsx
- drives.rs
- i18n.rs
- preview.rs
- foundation
- launcher.rs
- tray_window
- visibility.rs
- ShellAction
- File Structure
- pins
- pin_plan
- Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım
- src-tauri/src/lib.rs
- app_catalog.rs
- AppHandle
- extract-notes.mjs
- PinnedApp
- start
- ref_react
- devDependencies
- scripts
- dependencies
- vpn.rs
- WifiPage.tsx
- islandLogic.test.ts
- wallpaper.rs
- secure_store.rs
- connect_new_blocking
- volume.rs
- Client
- BluetoothPage.tsx
- super
- profile_xml
- dockItems.ts
- MacDows Taskbar Takeover Implementation Plan
- Dock Features Implementation Plan
- system.ts
- dockDrag.test.ts

## God Nodes (most connected - your core abstractions)
1. `Notch()` - 43 edges
2. `useT()` - 36 edges
3. `Dock()` - 27 edges
4. `TaskbarState` - 25 edges
5. `QuickSettingsScreen()` - 22 edges
6. `Window` - 18 edges
7. `UpdateStatus` - 17 edges
8. `initTheme()` - 16 edges
9. `WifiPage()` - 16 edges
10. `compilerOptions` - 16 edges

## Surprising Connections (you probably didn't know these)
- `4. Teknik tasarım` --references--> `BluetoothPage()`  [INFERRED]
  docs/superpowers/specs/2026-11-04-control-center-network-design.md → src/notch/BluetoothPage.tsx
- `State` --references--> `main()`  [INFERRED]
  docs/HANDOFF.md → scripts/build-installer.mjs
- `Task 2: Right-click menu, pin and unpin, new window, quit app` --references--> `LocalPin`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dock-features.md → src/bridge.ts
- `Task 2: Pure navigation logic (phase 2 core, TypeScript)` --references--> `FullscreenMode`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts
- `Task 2: Pure navigation logic (phase 2 core, TypeScript)` --references--> `ClickMode`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts

## Import Cycles
- None detected.

## Communities (124 total, 23 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.06
Nodes (37): Store apps use window icon (WhatsApp fix), APP, AppWindow, aumid_of(), BARS, class_of(), closed_windows_are_forgotten_and_reopened_go_to_the_end(), collect() (+29 more)

### Community 1 - "shelf.rs"
Cohesion: 0.12
Nodes (22): add_to_shelf(), clear_shelf(), CREATE_NO_WINDOW, current_session(), file_name(), get_shelf(), icon_for(), ICONS (+14 more)

### Community 2 - "DynamicIsland.tsx"
Cohesion: 0.09
Nodes (38): Task 6: Status, Music and Tools sub-features (phase 3), Props, silhouette(), SurfaceEdge, SurfaceShape(), addToShelf(), clearShelf(), getMedia() (+30 more)

### Community 3 - "media.rs"
Cohesion: 0.06
Nodes (37): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard), APP, attach_current() (+29 more)

### Community 4 - "package.json"
Cohesion: 0.12
Nodes (16): react, react-dom, @tauri-apps/api, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite (+8 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.07
Nodes (29): Tag-based signed release (GitHub Actions), app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+21 more)

### Community 6 - "settings.rs"
Cohesion: 0.11
Nodes (27): settings.json (corrupt file falls back to defaults), a_bad_island_value_does_not_reset_the_other_settings(), a_local_pin_round_trips(), a_partial_dock_object_is_filled_with_defaults(), a_v010_settings_file_without_update_fields_still_loads(), an_existing_island_object_wins_over_the_old_key(), an_old_file_without_island_gets_every_island_default(), an_old_settings_file_without_dock_loads() (+19 more)

### Community 7 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 8 - "dock_menu.rs"
Cohesion: 0.15
Nodes (9): change_icon(), CURRENT, DockMenuTarget, icon_key(), on_menu_event(), PREFIX, reset_icon(), run_action() (+1 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "bridge.ts"
Cohesion: 0.10
Nodes (24): Task 7: Quick settings sub-features (phase 3), btPair(), btPairingAnswer(), BtPairingRequest, btScanStart(), btScanStop(), Dock, MediaInfo (+16 more)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "bluetooth.rs"
Cohesion: 0.09
Nodes (14): address_hex(), batteries(), BATTERY_KEY, BATTERY_TIME_KEY, BtDevice, freshest(), get_bluetooth_devices(), kind_of() (+6 more)

### Community 17 - "system_theme.rs"
Cohesion: 0.07
Nodes (32): accent_from_registry(), get_system_accent_color(), abgr(), ACCENT, apply(), Backup, backup_path(), broadcast() (+24 more)

### Community 18 - "wifi.rs"
Cohesion: 0.16
Nodes (11): current_state(), is_enterprise(), merge(), net(), prints_the_wifi_networks(), read_networks(), sort_networks(), sorting_puts_connected_first_then_strongest_then_name() (+3 more)

### Community 19 - "updates.rs"
Cohesion: 0.07
Nodes (38): after_check(), available(), begin_check(), begin_install(), build_updater(), can_start(), check_for_update(), CHECK_INTERVAL_SECS (+30 more)

### Community 20 - "visualizer.rs"
Cohesion: 0.13
Nodes (14): a_louder_tone_shows_a_higher_bar(), a_steady_tone_lights_its_own_band(), analyze(), BANDS, bars_rise_at_once_and_fall_gradually(), capture_session(), CEIL_DB, EDGES (+6 more)

### Community 21 - "api.ts"
Cohesion: 0.08
Nodes (36): File Structure, Global Constraints, MacDows Installer and Logo Implementation Plan, Review Focus, Self-review, Task 1: Verify the real Windows facts before coding against them, Task 2: Logo sources and icon generation, Task 3: Pure installer logic (Rust, tests first) (+28 more)

### Community 22 - "tray_panel.rs"
Cohesion: 0.13
Nodes (25): activate(), automation(), BUSY, capture(), class_name(), close_flyout(), collect(), find_by_class() (+17 more)

### Community 23 - "Dock.tsx"
Cohesion: 0.14
Nodes (20): dockMenu(), Drive, focusWindow(), getCustomIcons(), launchNewApp(), launchPinned(), listDrives(), listPinned() (+12 more)

### Community 24 - "install.rs"
Cohesion: 0.08
Nodes (21): BUNDLED_VERSION, CREATE_NO_WINDOW, ErrorEvent, extract_payload(), extract_payload_works_with_non_ascii_dir(), install_state(), InstalledInfo, InstallInfo (+13 more)

### Community 25 - "Settings.tsx"
Cohesion: 0.13
Nodes (19): Task 4: UI, framer-motion, Settings, Visibility, RadialItem, RadialMenu(), CATEGORIES, Ctx (+11 more)

### Community 26 - "theme.ts"
Cohesion: 0.15
Nodes (21): applyBackdrop(), clearBackdrop(), STYLE_BLUR, usesBackdrop(), VARS, WallpaperBackdrop, wallpaperRect(), applyAccent() (+13 more)

### Community 27 - "taskbar_plan.rs"
Cohesion: 0.08
Nodes (20): a_restart_after_a_crash_keeps_the_stored_original(), guard_should_restore(), hidden(), hidden_icons_need_the_chevron_and_a_hidden_taskbar(), HidePlan, hiding_records_the_real_original(), needs_final_show(), needs_stale_restore() (+12 more)

### Community 28 - "installer/package.json"
Cohesion: 0.07
Nodes (28): dependencies, react, react-dom, @tauri-apps/api, devDependencies, @tauri-apps/cli, @types/react, @types/react-dom (+20 more)

### Community 30 - "logic.rs"
Cohesion: 0.11
Nodes (12): decide(), Decision, Fresh, Newer, Older, Reinstall, exit_code_message(), exit_code_zero_is_a_caller_bug() (+4 more)

### Community 31 - "island_settings.rs"
Cohesion: 0.08
Nodes (26): a_broken_order_is_repaired_to_every_screen_once(), click_and_fullscreen_modes_use_lowercase_names(), ClickMode, Player, Quick, Smart, defaults_match_the_spec(), FullscreenMode (+18 more)

### Community 32 - "hide"
Cohesion: 0.11
Nodes (11): Backend, hide(), hide_then_restore_returns_to_auto_hide_off(), HideError, StateNotSaved, hiding_again_after_a_crash_keeps_the_first_original(), MemStore, MovesTheWindowAway (+3 more)

### Community 33 - "Window"
Cohesion: 0.07
Nodes (24): MacDows.Setup, Application, App, Bar, Box, DonePanel, DoneText, DoneTitle (+16 more)

### Community 34 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+9 more)

### Community 35 - "taskbar.rs"
Cohesion: 0.08
Nodes (25): ABS_AUTOHIDE, battery_percent_from(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, current_status(), DETACHED_PROCESS, file_store_roundtrip_replace_clear_and_corrupt_file(), FileStore (+17 more)

### Community 36 - "installer/src-tauri/tauri.conf.json"
Cohesion: 0.12
Nodes (16): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+8 more)

### Community 37 - "MacDows Installer and Logo — Design"
Cohesion: 0.13
Nodes (14): Architecture, Build and release, Decisions (user), Errors, Goal, Install behavior, Logo, MacDows Installer and Logo — Design (+6 more)

### Community 38 - "make-icons.mjs"
Cohesion: 0.13
Nodes (13): png-to-ico, @resvg/resvg-js, appIcons, assets, icoSizes, installerIcons, main, pngFor() (+5 more)

### Community 39 - "quick_settings.rs"
Cohesion: 0.08
Nodes (27): airplane_on(), airplane_tile(), banners_on(), clamp_percent(), CREATE_NO_WINDOW, get_brightness(), get_quick_settings(), has_battery() (+19 more)

### Community 40 - "rig"
Cohesion: 0.13
Nodes (7): FakeBackend, FakeStore, hide_then_restore_returns_to_auto_hide_on_when_it_was_on(), nothing_is_hidden_when_the_state_cannot_be_saved(), restore_without_state_shows_the_window_and_leaves_auto_hide_alone(), rig(), the_state_is_saved_before_anything_is_hidden()

### Community 41 - "Dynamic Island Settings Implementation Plan"
Cohesion: 0.25
Nodes (7): Dynamic Island Settings Implementation Plan, Global Constraints, Review Focus, Self-Review, Status (2026-11-04, evening), Where the spec is silent (interpretations; no feature of the spec is dropped), ToolsCfg

### Community 42 - "MacDows Taskbar Takeover — Design"
Cohesion: 0.15
Nodes (12): Behavior details, Decisions (user), Errors, Goal, How the taskbar is hidden, MacDows Taskbar Takeover — Design, Risks, Scope (+4 more)

### Community 43 - "bluetooth_pairing.rs"
Cohesion: 0.08
Nodes (19): ANSWER, bool_prop(), bt_pair(), bt_scan_start(), bt_unpair(), FILTER, Found, LISTENING (+11 more)

### Community 44 - "Handoff — where MacDows stands (2026-11-02)"
Cohesion: 0.20
Nodes (9): Decisions to keep, Handoff — where MacDows stands (2026-11-02), Knowledge graph, Known gaps (deferred), Release secrets (important), Roadmap, Run it, Start and Search (2026-11-03) (+1 more)

### Community 45 - "Dinamik Ada ayarları: tasarım"
Cohesion: 0.18
Nodes (10): 1. Amaç, 2.1 Ekranlar, 2.2 Gezinme, 2.3 Görünürlük ve yerleşim, 2. Ayar modeli, 3. Davranış kuralları, 5. Aşamalar, 6. Kapsam dışı (+2 more)

### Community 46 - "run_install"
Cohesion: 0.31
Nodes (10): apply_autostart_choice(), emit_stage(), fail(), launch_app(), run_install(), run_value(), start_install(), system_locale() (+2 more)

### Community 47 - "build-installer.mjs"
Cohesion: 0.16
Nodes (6): findPayload(), main(), root, run(), sumsLine(), root

### Community 48 - "pinned_apps.rs"
Cohesion: 0.10
Nodes (19): a_store_app_named_twice_counts_once_at_its_first_mention(), APPS_FOLDER, blob(), explorer_is_found_by_its_short_name_too(), find(), find_ignore_case(), is_aumid(), launch_new() (+11 more)

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
Cohesion: 0.07
Nodes (20): APP, apply(), auto_repeat_is_swallowed_without_firing_again(), decide(), Decision, HELD, hook_proc(), HOOK_THREAD (+12 more)

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
Nodes (29): Task 8: Notification sub-features (phase 3, Rust + UI), clearNotificationHistory(), getNotificationHistory(), getSettings(), Notice, NotifCfg, onNotificationHistoryChanged(), onSettingsChanged() (+21 more)

### Community 66 - "fullscreen.rs"
Cohesion: 0.10
Nodes (11): Event-driven, no polling for windows and media, APP, class_of(), covers_monitor(), LAST, on_event(), probe(), should_hide() (+3 more)

### Community 69 - "QuickSettings.tsx"
Cohesion: 0.10
Nodes (31): BtDevice, getBrightness(), getQuickSettings(), getVolume(), getVpnState(), openMacdowsSettings(), QuickKey, quickLaunch() (+23 more)

### Community 70 - "custom_icons.rs"
Cohesion: 0.13
Nodes (17): a_file_that_is_not_an_image_is_rejected(), a_missing_file_is_rejected(), a_picture_is_saved_as_a_128_png(), a_wide_image_is_fitted_with_transparent_bars_above_and_below(), fit_to_square(), get_custom_icons(), icon_file_name(), icons_dir() (+9 more)

### Community 71 - "File Structure"
Cohesion: 0.25
Nodes (8): File Structure, Task 1: Pure plans and key steps (tests first), Task 2: Hide and restore over traits (tests first), Task 3: Real Windows backend, state file and shell actions, Task 4: Guard binary, CLI flags and packaging, Task 5: Wiring and settings, Task 7: Real runs on this PC, Task 8: Docs

### Community 72 - "File Structure"
Cohesion: 0.20
Nodes (10): File Structure, Task 1: Dock settings model, Task 3: Drag to reorder pins (saved order), Task 4: Custom icons, Task 5: Drives, Task 6: Win+1..9, Task 8: Add app panel, Task 9: Settings switches, texts, changelog (+2 more)

### Community 73 - "MenuAction"
Cohesion: 0.11
Nodes (16): ALL_ACTIONS, dock_menu(), menu_plan(), MenuAction, AddApp, ChangeIcon, MacdowsSettings, NewWindow (+8 more)

### Community 74 - "IslandSettings.tsx"
Cohesion: 0.12
Nodes (33): Task 3: Settings page skeleton, visibility/scale/fullscreen moved (phase 1, UI), Task 4: Screens and navigation sections in the settings page (phase 2, UI), Task 9: Shelf sub-features (phase 3, Rust + UI), 4. Arayüz, STYLE_OPACITY, THEME_STYLES, ClickMode, FullscreenMode (+25 more)

### Community 75 - "drives.rs"
Cohesion: 0.11
Nodes (14): Drive, DRIVE_FIXED, DRIVE_REMOVABLE, enabled(), kind_of(), letters(), list_drives(), open_drive() (+6 more)

### Community 76 - "i18n.rs"
Cohesion: 0.18
Nodes (3): Languages: English and Turkish, taskbar_notice(), the_first_run_notice_explains_how_to_undo_it()

### Community 77 - "preview.rs"
Cohesion: 0.07
Nodes (33): CARD_H, CARD_W, CLOSE_DELAY_MS, close_panel(), close_preview(), DOCK_HOVER, fit_size(), GAP (+25 more)

### Community 78 - "foundation"
Cohesion: 0.17
Nodes (10): append_system_items(), Choice, Dock, None, System, classify(), popup(), send_system_command() (+2 more)

### Community 79 - "launcher.rs"
Cohesion: 0.11
Nodes (18): BOOTSTRAPPER_URL, CLIENT_KEY, CREATE_NO_WINDOW, download(), download_args(), download_command_follows_redirects_and_names_the_target(), ensure_webview2(), first_pv() (+10 more)

### Community 80 - "tray_window"
Cohesion: 0.18
Nodes (8): appbar_state(), chevron_present_uia(), is_chevron_name(), set_appbar_state(), start_watchdog(), tray_window(), watcher_proc(), WinBackend

### Community 82 - "visibility.rs"
Cohesion: 0.11
Nodes (12): Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, foreground_exe(), hidden_in_fullscreen(), hold_notch(), inp(), Inputs, is_shell_menu() (+4 more)

### Community 83 - "ShellAction"
Cohesion: 0.17
Nodes (12): hidden_icons_jump_to_the_chevron_with_home_before_pressing_enter(), key_steps(), KeyStep, Chord, Wait, ShellAction, HiddenIcons, Notifications (+4 more)

### Community 84 - "File Structure"
Cohesion: 0.17
Nodes (11): Control Center Network Pages Implementation Plan, File Structure, Global Constraints, Review Focus, Self-Review, Task 1: Wi-Fi backend: adapters, scan, list, profile XML (no connecting yet), Task 2: Wi-Fi page: list, connect to saved or open networks, disconnect, forget, Task 3: Password entry, new secured networks, hidden networks, keyboard focus (+3 more)

### Community 86 - "pins"
Cohesion: 0.21
Nodes (12): favorites_blob(), fingerprint(), is_listed(), is_shortcut(), Pin, Shortcut, Store, pinned_dir() (+4 more)

### Community 87 - "pin_plan"
Cohesion: 0.24
Nodes (10): a_local_pin_windows_now_has_is_not_listed_twice(), local_pins_follow_the_windows_pins(), merge_local(), norm(), pin_plan(), PinPlan, AddLocal, AlreadyShown (+2 more)

### Community 89 - "Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım"
Cohesion: 0.15
Nodes (12): 1. Amaç, 2.1 Wi-Fi sayfası, 2.2 Şifre ve klavye odağı, 2.3 Bluetooth sayfası, 2.4 VPN sayfası, 2. Arayüz, 3. Ayarlar, 4. Teknik tasarım (+4 more)

### Community 91 - "src-tauri/src/lib.rs"
Cohesion: 0.12
Nodes (25): Single instance: second launch opens Settings, activate_tray_item(), apply_replace_taskbar(), AppSettings, bring_to_front(), build_tray(), close_tray(), close_tray_panel() (+17 more)

### Community 93 - "app_catalog.rs"
Cohesion: 0.11
Nodes (24): app(), CACHE, CACHE_FOR, CatalogApp, close_add_app_panel(), collect_shortcuts(), dedupe(), duplicates_by_program_or_app_id_keep_the_first() (+16 more)

### Community 94 - "AppHandle"
Cohesion: 0.24
Nodes (9): pin_app(), pin_source(), remove_pin(), set_pinned_order(), unpin_plan(), UnpinPlan, DeleteLocal, HideInDock (+1 more)

### Community 95 - "extract-notes.mjs"
Cohesion: 0.50
Nodes (3): end, lines, start

### Community 96 - "PinnedApp"
Cohesion: 0.29
Nodes (10): app(), apply_order(), apply_order_ignores_case_and_leaves_an_empty_order_alone(), apply_order_ignores_pins_that_are_gone(), apply_order_keeps_the_saved_order_and_appends_new_pins(), fill_local_icons(), list_pinned(), PinnedApp (+2 more)

### Community 97 - "start"
Cohesion: 0.24
Nodes (10): MON, Rect, cursor(), overlap_probe(), point_in_rect(), rect_intersects(), rect_of(), reveal_band() (+2 more)

### Community 98 - "ref_react"
Cohesion: 0.12
Nodes (20): Task 7: Window previews and count badge, enableBackdrop(), initTheme(), CatalogApp, closeAddAppPanel(), closePreview(), getCatalog(), onPinnedChanged() (+12 more)

### Community 102 - "devDependencies"
Cohesion: 0.22
Nodes (9): devDependencies, png-to-ico, @resvg/resvg-js, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite (+1 more)

### Community 104 - "scripts"
Cohesion: 0.25
Nodes (8): scripts, build, dev, icons, preview, tauri, test:logic, test:scripts

### Community 106 - "dependencies"
Cohesion: 0.29
Nodes (7): dependencies, @crabnebula/tauri-plugin-drag, framer-motion, react, react-dom, @tauri-apps/api, @tauri-apps/plugin-opener

### Community 108 - "vpn.rs"
Cohesion: 0.14
Nodes (16): app_vpn_active(), connect_blocking(), connection_state(), connections(), current_state(), disconnect_blocking(), entry_names(), is_tunnel_adapter() (+8 more)

### Community 109 - "WifiPage.tsx"
Cohesion: 0.20
Nodes (18): getWifiState(), wifiConnect(), wifiConnectNew(), wifiDisconnect(), wifiForget(), WifiNetwork, wifiScan(), WifiState (+10 more)

### Community 110 - "islandLogic.test.ts"
Cohesion: 0.24
Nodes (16): File Structure, Task 10: Reset buttons, texts, polish (phase 4), Task 1: Island settings model, migration, normalization (phase 1, Rust), Task 2: Pure navigation logic (phase 2 core, TypeScript), Task 5: The island obeys screen and navigation settings (phase 2, runtime), Island, ScreenId, defaultIsland() (+8 more)

### Community 111 - "wallpaper.rs"
Cohesion: 0.21
Nodes (10): BACKDROP_SIZE, CACHE, CACHE_LIMIT, CachedImage, get_wallpaper_backdrop(), load_blurred(), read_desktop_string(), wallpaper_fit() (+2 more)

### Community 112 - "secure_store.rs"
Cohesion: 0.29
Nodes (8): blob(), MAGIC, open(), plain_files_and_damaged_data_are_not_opened(), seal(), take(), the_sealed_bytes_do_not_contain_the_text(), what_is_sealed_can_be_opened_again()

### Community 113 - "connect_new_blocking"
Cohesion: 0.29
Nodes (11): connect_blocking(), connect_new_blocking(), connect_profile(), error_key(), failure_key(), forget_blocking(), is_connected_to(), wide() (+3 more)

### Community 114 - "volume.rs"
Cohesion: 0.42
Nodes (7): clamp_level(), endpoint(), get_volume(), read(), set_volume(), Volume, write()

### Community 115 - "Client"
Cohesion: 0.33
Nodes (8): Client, current_connection(), disconnect_blocking(), find_network(), Found, interfaces(), open(), request_scan()

### Community 116 - "BluetoothPage.tsx"
Cohesion: 0.33
Nodes (7): BtKind, btUnpair(), getBluetoothDevices(), setBluetoothConnection(), Battery(), BluetoothPage(), KIND_ICON

### Community 117 - "super"
Cohesion: 0.25
Nodes (5): Cli, Guard, Normal, RestoreTaskbar, parse()

### Community 118 - "profile_xml"
Cohesion: 0.22
Nodes (9): Auth, Open, Wpa2Personal, Wpa3Personal, open_profile_has_no_key(), parse_auth(), profile_xml(), wpa2_profile_has_the_key() (+1 more)

### Community 119 - "dockItems.ts"
Cohesion: 0.43
Nodes (7): Task 2: Right-click menu, pin and unpin, new window, quit app, AppWindow, PinnedApp, belongs(), DockEntry, mergeDock(), same()

### Community 120 - "MacDows Taskbar Takeover Implementation Plan"
Cohesion: 0.29
Nodes (6): Global Constraints, MacDows Taskbar Takeover Implementation Plan, Review Focus, Self-review, SystemButton(), shellAction

### Community 121 - "Dock Features Implementation Plan"
Cohesion: 0.33
Nodes (5): Dock Features Implementation Plan, Global Constraints, Review Focus, Self-Review, DockMenuTarget

### Community 122 - "system.ts"
Cohesion: 0.50
Nodes (4): Task 6: UI and uninstall hook, getSystemStatus(), SystemStatus, useSystemStatus()

## Knowledge Gaps
- **512 isolated node(s):** `PAIRED_KEY`, `NAME_KEY`, `FILTER`, `Found`, `Lost` (+507 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 892 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **23 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Dock (running apps, stable order)` connect `media.rs` to `windows_list.rs`, `Dock.tsx`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `PAIRED_KEY`, `NAME_KEY`, `FILTER` to the rest of the system?**
  _512 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05786090005844535 - nodes in this community are weakly interconnected._
- **Why does `Visibility modes Fixed/Smart/Peek` connect `visibility.rs` to `Settings.tsx`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Should `shelf.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.12096774193548387 - nodes in this community are weakly interconnected._
- **Why does `Languages: English and Turkish` connect `i18n.rs` to `useT`?**
  _High betweenness centrality (0.030) - this node is a cross-community bridge._
- **Should `DynamicIsland.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09191583610188261 - nodes in this community are weakly interconnected._