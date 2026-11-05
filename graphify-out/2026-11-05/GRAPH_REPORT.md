# Graph Report - MacDows  (2026-11-05)

## Corpus Check
- 143 files · ~121,160 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 25 file(s) not represented in the graph (top: .css 10, (none) 6, .ico 3)

## Summary
- 2349 nodes · 4338 edges · 135 communities (96 shown, 39 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 146 edges (avg confidence: 0.93)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `438b8f11`
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
- drives.rs
- default.json
- wifi.rs
- macdows
- system_theme.rs
- custom_icons.rs
- updates.rs
- vpn.rs
- api.ts
- tray_panel.rs
- Dock.tsx
- install.rs
- bridge.ts
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
- bluetooth.rs
- MacDows Taskbar Takeover — Design
- bluetooth_pairing.rs
- build-installer.mjs
- Dinamik Ada ayarları: tasarım
- run_install
- MenuAction
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
- Dynamic Island Settings Implementation Plan
- File Structure
- start_watcher
- collect
- IslandSettings.tsx
- pin_plan
- super
- preview.rs
- window_menu.rs
- launcher.rs
- WifiPage.tsx
- visibility.rs
- ShellAction
- File Structure
- StatusCfg
- pins
- Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım
- src-tauri/src/lib.rs
- Durum widget'ları (Nectar'daki gibi), tasarım
- app_catalog.rs
- devDependencies
- TaskbarState
- scripts
- Handoff — where MacDows stands (2026-11-02)
- Settings.tsx
- StatusWidgets.tsx
- visualizer.rs
- dependencies
- Mutex
- wallpaper.rs
- StatusWidgetsEditor.tsx
- UnpinPlan
- volume.rs
- UpdateEffect
- AtomicBool
- initTheme
- Dock Features Implementation Plan
- extract-notes.mjs

## God Nodes (most connected - your core abstractions)
1. `Notch()` - 49 edges
2. `useT()` - 36 edges
3. `Dock()` - 27 edges
4. `QuickSettingsScreen()` - 23 edges
5. `TaskbarState` - 22 edges
6. `Window` - 18 edges
7. `UpdateStatus` - 17 edges
8. `initTheme()` - 16 edges
9. `WifiPage()` - 16 edges
10. `Notice` - 16 edges

## Surprising Connections (you probably didn't know these)
- `Task 8: Notification sub-features (phase 3, Rust + UI)` --references--> `NotifCfg`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts
- `Task 9: Shelf sub-features (phase 3, Rust + UI)` --references--> `ShelfCfg`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/bridge.ts
- `5. Ayarlar modeli` --references--> `StatusCfg`  [INFERRED]
  docs/superpowers/specs/2026-11-04-status-widgets-design.md → src/bridge.ts
- `Task 3: Settings page skeleton, visibility/scale/fullscreen moved (phase 1, UI)` --references--> `Category`  [INFERRED]
  docs/superpowers/plans/2026-11-04-dynamic-island-settings.md → src/settings/Settings.tsx
- `State` --references--> `main()`  [INFERRED]
  docs/HANDOFF.md → scripts/build-installer.mjs

## Import Cycles
- None detected.

## Communities (135 total, 39 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.07
Nodes (23): Store apps use window icon (WhatsApp fix), APP, AppWindow, BARS, closed_windows_are_forgotten_and_reopened_go_to_the_end(), enumerate(), EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED (+15 more)

### Community 1 - "shelf.rs"
Cohesion: 0.12
Nodes (22): add_to_shelf(), clear_shelf(), CREATE_NO_WINDOW, current_session(), file_name(), get_shelf(), icon_for(), ICONS (+14 more)

### Community 2 - "DynamicIsland.tsx"
Cohesion: 0.07
Nodes (53): Task 2: Pure navigation logic (phase 2 core, TypeScript), Task 5: The island obeys screen and navigation settings (phase 2, runtime), Task 6: Status, Music and Tools sub-features (phase 3), Props, silhouette(), SurfaceEdge, SurfaceShape(), addToShelf() (+45 more)

### Community 3 - "media.rs"
Cohesion: 0.06
Nodes (37): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard), APP, attach_current() (+29 more)

### Community 4 - "package.json"
Cohesion: 0.12
Nodes (15): react, react-dom, @tauri-apps/api, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite (+7 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.07
Nodes (29): Tag-based signed release (GitHub Actions), app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+21 more)

### Community 6 - "settings.rs"
Cohesion: 0.12
Nodes (27): settings.json (corrupt file falls back to defaults), a_bad_island_value_does_not_reset_the_other_settings(), a_local_pin_round_trips(), a_partial_dock_object_is_filled_with_defaults(), a_v010_settings_file_without_update_fields_still_loads(), an_existing_island_object_wins_over_the_old_key(), an_old_file_without_island_gets_every_island_default(), an_old_settings_file_without_dock_loads() (+19 more)

### Community 7 - "compilerOptions"
Cohesion: 0.10
Nodes (19): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+11 more)

### Community 8 - "dock_menu.rs"
Cohesion: 0.15
Nodes (11): change_icon(), DockMenuTarget, icon_key(), on_menu_event(), PREFIX, reset_icon(), run_action(), warn() (+3 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "drives.rs"
Cohesion: 0.13
Nodes (12): Drive, DRIVE_FIXED, DRIVE_REMOVABLE, enabled(), kind_of(), letters(), list_drives(), open_drive() (+4 more)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "wifi.rs"
Cohesion: 0.07
Nodes (48): Auth, auth_name(), Open, Wpa2Personal, Wpa3Personal, Client, connect_blocking(), connect_new_blocking() (+40 more)

### Community 17 - "system_theme.rs"
Cohesion: 0.07
Nodes (31): accent_from_registry(), get_system_accent_color(), abgr(), ACCENT, apply(), Backup, backup_path(), broadcast() (+23 more)

### Community 18 - "custom_icons.rs"
Cohesion: 0.13
Nodes (17): a_file_that_is_not_an_image_is_rejected(), a_missing_file_is_rejected(), a_picture_is_saved_as_a_128_png(), a_wide_image_is_fitted_with_transparent_bars_above_and_below(), fit_to_square(), get_custom_icons(), icon_file_name(), icons_dir() (+9 more)

### Community 19 - "updates.rs"
Cohesion: 0.07
Nodes (38): after_check(), available(), begin_check(), begin_install(), build_updater(), can_start(), check_for_update(), CHECK_INTERVAL_SECS (+30 more)

### Community 20 - "vpn.rs"
Cohesion: 0.06
Nodes (28): TOAST_KEY, cpu_percent(), cpu_times(), disk_percent(), ft(), get_system_stats(), net_counters(), Prev (+20 more)

### Community 21 - "api.ts"
Cohesion: 0.09
Nodes (35): File Structure, Global Constraints, MacDows Installer and Logo Implementation Plan, Review Focus, Self-review, Task 1: Verify the real Windows facts before coding against them, Task 2: Logo sources and icon generation, Task 3: Pure installer logic (Rust, tests first) (+27 more)

### Community 22 - "tray_panel.rs"
Cohesion: 0.13
Nodes (25): activate(), automation(), BUSY, capture(), class_name(), close_flyout(), collect(), find_by_class() (+17 more)

### Community 23 - "Dock.tsx"
Cohesion: 0.07
Nodes (40): File Structure, Task 1: Dock settings model, Task 2: Right-click menu, pin and unpin, new window, quit app, Task 3: Drag to reorder pins (saved order), Task 4: Custom icons, Task 5: Drives, Task 6: Win+1..9, Task 8: Add app panel (+32 more)

### Community 24 - "install.rs"
Cohesion: 0.08
Nodes (22): BUNDLED_VERSION, CREATE_NO_WINDOW, ErrorEvent, extract_payload(), extract_payload_works_with_non_ascii_dir(), install_state(), InstalledInfo, InstallInfo (+14 more)

### Community 25 - "bridge.ts"
Cohesion: 0.10
Nodes (25): Task 7: Quick settings sub-features (phase 3), btPair(), btPairingAnswer(), BtPairingRequest, btScanStart(), btScanStop(), Dock, MediaInfo (+17 more)

### Community 26 - "theme.ts"
Cohesion: 0.13
Nodes (24): applyBackdrop(), clearBackdrop(), enableBackdrop(), STYLE_BLUR, usesBackdrop(), VARS, WallpaperBackdrop, wallpaperRect() (+16 more)

### Community 27 - "taskbar_plan.rs"
Cohesion: 0.09
Nodes (12): a_restart_after_a_crash_keeps_the_stored_original(), hidden(), hidden_icons_need_the_chevron_and_a_hidden_taskbar(), HidePlan, hiding_records_the_real_original(), plan_hide(), plan_restore(), restore_puts_back_each_original() (+4 more)

### Community 28 - "installer/package.json"
Cohesion: 0.07
Nodes (28): dependencies, react, react-dom, @tauri-apps/api, devDependencies, @tauri-apps/cli, @types/react, @types/react-dom (+20 more)

### Community 30 - "logic.rs"
Cohesion: 0.11
Nodes (12): decide(), Decision, Fresh, Newer, Older, Reinstall, exit_code_message(), exit_code_zero_is_a_caller_bug() (+4 more)

### Community 31 - "island_settings.rs"
Cohesion: 0.07
Nodes (35): a_broken_order_is_repaired_to_every_screen_once(), click_and_fullscreen_modes_use_lowercase_names(), ClickMode, Player, Quick, Smart, defaults_match_the_spec(), FullscreenMode (+27 more)

### Community 32 - "hide"
Cohesion: 0.13
Nodes (10): Backend, hide(), hide_then_restore_returns_to_auto_hide_off(), hide_then_restore_returns_to_auto_hide_on_when_it_was_on(), HideError, StateNotSaved, hiding_again_after_a_crash_keeps_the_first_original(), MovesTheWindowAway (+2 more)

### Community 33 - "Window"
Cohesion: 0.07
Nodes (24): MacDows.Setup, Application, App, Bar, Box, DonePanel, DoneText, DoneTitle (+16 more)

### Community 34 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+9 more)

### Community 35 - "taskbar.rs"
Cohesion: 0.06
Nodes (35): APP, ABS_AUTOHIDE, appbar_state(), battery_percent_from(), chevron_present_uia(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, current_status() (+27 more)

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
Cohesion: 0.18
Nodes (5): FakeBackend, FakeStore, nothing_is_hidden_when_the_state_cannot_be_saved(), rig(), the_state_is_saved_before_anything_is_hidden()

### Community 41 - "bluetooth.rs"
Cohesion: 0.09
Nodes (14): address_hex(), batteries(), BATTERY_KEY, BATTERY_TIME_KEY, BtDevice, freshest(), get_bluetooth_devices(), kind_of() (+6 more)

### Community 42 - "MacDows Taskbar Takeover — Design"
Cohesion: 0.15
Nodes (12): Behavior details, Decisions (user), Errors, Goal, How the taskbar is hidden, MacDows Taskbar Takeover — Design, Risks, Scope (+4 more)

### Community 43 - "bluetooth_pairing.rs"
Cohesion: 0.10
Nodes (12): bt_pair(), bt_unpair(), FILTER, Found, Lost, NAME_KEY, pair_blocking(), pair_error() (+4 more)

### Community 44 - "build-installer.mjs"
Cohesion: 0.19
Nodes (7): Task 8: Build script, release workflow, docs, findPayload(), main(), root, run(), sumsLine(), root

### Community 45 - "Dinamik Ada ayarları: tasarım"
Cohesion: 0.17
Nodes (11): 1. Amaç, 2.1 Ekranlar, 2.2 Gezinme, 2.3 Görünürlük ve yerleşim, 2. Ayar modeli, 3. Davranış kuralları, 4. Arayüz, 5. Aşamalar (+3 more)

### Community 46 - "run_install"
Cohesion: 0.35
Nodes (9): apply_autostart_choice(), emit_stage(), fail(), launch_app(), run_install(), start_install(), system_locale(), write_log() (+1 more)

### Community 47 - "MenuAction"
Cohesion: 0.11
Nodes (16): ALL_ACTIONS, dock_menu(), menu_plan(), MenuAction, AddApp, ChangeIcon, MacdowsSettings, NewWindow (+8 more)

### Community 48 - "pinned_apps.rs"
Cohesion: 0.09
Nodes (23): a_store_app_named_twice_counts_once_at_its_first_mention(), apply_order(), apply_order_ignores_case_and_leaves_an_empty_order_alone(), apply_order_ignores_pins_that_are_gone(), apply_order_keeps_the_saved_order_and_appends_new_pins(), APPS_FOLDER, blob(), explorer_is_found_by_its_short_name_too() (+15 more)

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
Cohesion: 0.06
Nodes (20): APP, apply(), auto_repeat_is_swallowed_without_firing_again(), decide(), Decision, HELD, hook_proc(), HOOK_THREAD (+12 more)

### Community 55 - "installer/src-tauri/capabilities/default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 56 - "notifications.rs"
Cohesion: 0.10
Nodes (29): a_cleared_history_does_not_take_the_old_ones_back(), after_cutoff(), BANNER_HOLD_MS, clear_notification_history(), cleared_at(), cutoff_path(), fresh(), get_notification_history() (+21 more)

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

### Community 66 - "fullscreen.rs"
Cohesion: 0.09
Nodes (14): Event-driven, no polling for windows and media, APP, class_of(), covers_monitor(), LAST, MON, on_event(), probe() (+6 more)

### Community 69 - "QuickSettings.tsx"
Cohesion: 0.08
Nodes (38): BtDevice, BtKind, btUnpair(), getBluetoothDevices(), getBrightness(), getQuickSettings(), getVolume(), getVpnState() (+30 more)

### Community 70 - "Dynamic Island Settings Implementation Plan"
Cohesion: 0.25
Nodes (7): Dynamic Island Settings Implementation Plan, Global Constraints, Review Focus, Self-Review, Status (2026-11-04, evening), Where the spec is silent (interpretations; no feature of the spec is dropped), ToolsCfg

### Community 71 - "File Structure"
Cohesion: 0.14
Nodes (13): File Structure, Global Constraints, MacDows Taskbar Takeover Implementation Plan, Review Focus, Self-review, Task 1: Pure plans and key steps (tests first), Task 2: Hide and restore over traits (tests first), Task 3: Real Windows backend, state file and shell actions (+5 more)

### Community 72 - "start_watcher"
Cohesion: 0.29
Nodes (7): bool_prop(), bt_scan_start(), name_prop(), note_paired(), start_watcher(), update_effect(), visible_in_scan()

### Community 73 - "collect"
Cohesion: 0.17
Nodes (14): aumid_of(), class_of(), collect(), collect_rects(), exe_of(), focus_window(), hicon_to_png_base64(), icon_png_base64() (+6 more)

### Community 74 - "IslandSettings.tsx"
Cohesion: 0.15
Nodes (31): File Structure, Task 10: Reset buttons, texts, polish (phase 4), Task 1: Island settings model, migration, normalization (phase 1, Rust), Task 3: Settings page skeleton, visibility/scale/fullscreen moved (phase 1, UI), Task 4: Screens and navigation sections in the settings page (phase 2, UI), Task 8: Notification sub-features (phase 3, Rust + UI), Task 9: Shelf sub-features (phase 3, Rust + UI), framer-motion (+23 more)

### Community 75 - "pin_plan"
Cohesion: 0.17
Nodes (18): a_local_pin_windows_now_has_is_not_listed_twice(), app(), fill_local_icons(), list_pinned(), local_pins_follow_the_windows_pins(), merge_local(), norm(), pin_app() (+10 more)

### Community 76 - "super"
Cohesion: 0.08
Nodes (16): Languages: English and Turkish, Cli, Guard, Normal, RestoreTaskbar, parse(), taskbar_notice(), the_first_run_notice_explains_how_to_undo_it() (+8 more)

### Community 77 - "preview.rs"
Cohesion: 0.07
Nodes (33): CARD_H, CARD_W, CLOSE_DELAY_MS, close_panel(), close_preview(), DOCK_HOVER, fit_size(), GAP (+25 more)

### Community 78 - "window_menu.rs"
Cohesion: 0.15
Nodes (11): append_system_items(), Choice, Dock, None, System, classify(), needs_focus(), popup() (+3 more)

### Community 79 - "launcher.rs"
Cohesion: 0.11
Nodes (18): BOOTSTRAPPER_URL, CLIENT_KEY, CREATE_NO_WINDOW, download(), download_args(), download_command_follows_redirects_and_names_the_target(), ensure_webview2(), first_pv() (+10 more)

### Community 80 - "WifiPage.tsx"
Cohesion: 0.19
Nodes (19): getWifiState(), wifiConnect(), wifiConnectNew(), wifiDisconnect(), wifiForget(), WifiNetwork, wifiScan(), WifiState (+11 more)

### Community 82 - "visibility.rs"
Cohesion: 0.09
Nodes (22): Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, cursor(), foreground_exe(), hidden_in_fullscreen(), hold_notch(), hold_notch_for_typing(), inp() (+14 more)

### Community 83 - "ShellAction"
Cohesion: 0.17
Nodes (12): hidden_icons_jump_to_the_chevron_with_home_before_pressing_enter(), key_steps(), KeyStep, Chord, Wait, ShellAction, HiddenIcons, Notifications (+4 more)

### Community 84 - "File Structure"
Cohesion: 0.17
Nodes (11): Control Center Network Pages Implementation Plan, File Structure, Global Constraints, Review Focus, Self-Review, Task 1: Wi-Fi backend: adapters, scan, list, profile XML (no connecting yet), Task 2: Wi-Fi page: list, connect to saved or open networks, disconnect, forget, Task 3: Password entry, new secured networks, hidden networks, keyboard focus (+3 more)

### Community 86 - "StatusCfg"
Cohesion: 0.15
Nodes (13): Global Constraints, Review Focus, Status Widgets Implementation Plan, Task 2: System stats command, Task 5: Settings editor (drag and drop, city, unit, threshold), setSetting(), StatusCfg, SystemStats (+5 more)

### Community 87 - "pins"
Cohesion: 0.21
Nodes (12): favorites_blob(), fingerprint(), is_listed(), is_shortcut(), Pin, Shortcut, Store, pinned_dir() (+4 more)

### Community 89 - "Kontrol Merkezi: Wi-Fi, Bluetooth ve VPN sayfaları, tasarım"
Cohesion: 0.15
Nodes (12): 1. Amaç, 2.1 Wi-Fi sayfası, 2.2 Şifre ve klavye odağı, 2.3 Bluetooth sayfası, 2.4 VPN sayfası, 2. Arayüz, 3. Ayarlar, 4. Teknik tasarım (+4 more)

### Community 91 - "src-tauri/src/lib.rs"
Cohesion: 0.12
Nodes (25): Single instance: second launch opens Settings, activate_tray_item(), apply_replace_taskbar(), AppSettings, bring_to_front(), build_tray(), close_tray(), close_tray_panel() (+17 more)

### Community 92 - "Durum widget'ları (Nectar'daki gibi), tasarım"
Cohesion: 0.22
Nodes (8): 1. Amaç, 2. Yerleşim ve görünüm, 3. Veri, 4. Gizlilik, 5. Ayarlar modeli, 6. Ayarlar arayüzü, 9. Riskler, Durum widget'ları (Nectar'daki gibi), tasarım

### Community 93 - "app_catalog.rs"
Cohesion: 0.12
Nodes (24): app(), CACHE, CACHE_FOR, CatalogApp, close_add_app_panel(), collect_shortcuts(), dedupe(), duplicates_by_program_or_app_id_keep_the_first() (+16 more)

### Community 94 - "devDependencies"
Cohesion: 0.22
Nodes (9): devDependencies, png-to-ico, @resvg/resvg-js, @tauri-apps/cli, @types/react, @types/react-dom, typescript, vite (+1 more)

### Community 95 - "TaskbarState"
Cohesion: 0.14
Nodes (10): guard_should_restore(), MemStore, needs_final_show(), needs_stale_restore(), restore_ends_with_a_visible_taskbar_even_if_explorer_hides_it_again(), state_belongs_to(), state_from_json(), Store (+2 more)

### Community 96 - "scripts"
Cohesion: 0.25
Nodes (8): scripts, build, dev, icons, preview, tauri, test:logic, test:scripts

### Community 97 - "Handoff — where MacDows stands (2026-11-02)"
Cohesion: 0.20
Nodes (9): Decisions to keep, Handoff — where MacDows stands (2026-11-02), Knowledge graph, Known gaps (deferred), Release secrets (important), Roadmap, Run it, Start and Search (2026-11-03) (+1 more)

### Community 98 - "Settings.tsx"
Cohesion: 0.14
Nodes (17): Task 4: UI, Settings, Visibility, CATEGORIES, Category, Ctx, Item, ITEMS (+9 more)

### Community 102 - "StatusWidgets.tsx"
Cohesion: 0.13
Nodes (35): File Structure, Task 1: Settings model, normalization, migration, Task 3: Widget logic (pure TS), Task 4: Widget row in the island, Task 6: Strings, changelog, final pass, 7. Modüller, 8. Test, getSystemStats() (+27 more)

### Community 104 - "visualizer.rs"
Cohesion: 0.13
Nodes (14): a_louder_tone_shows_a_higher_bar(), a_steady_tone_lights_its_own_band(), analyze(), BANDS, bars_rise_at_once_and_fall_gradually(), capture_session(), CEIL_DB, EDGES (+6 more)

### Community 106 - "dependencies"
Cohesion: 0.29
Nodes (7): dependencies, @crabnebula/tauri-plugin-drag, framer-motion, react, react-dom, @tauri-apps/api, @tauri-apps/plugin-opener

### Community 108 - "Mutex"
Cohesion: 0.29
Nodes (4): ANSWER, KNOWN_PAIRED, WATCHER, CURRENT

### Community 109 - "wallpaper.rs"
Cohesion: 0.21
Nodes (10): BACKDROP_SIZE, CACHE, CACHE_LIMIT, CachedImage, get_wallpaper_backdrop(), load_blurred(), read_desktop_string(), wallpaper_fit() (+2 more)

### Community 112 - "StatusWidgetsEditor.tsx"
Cohesion: 0.29
Nodes (7): WidgetId, CityResult, DEFAULT_WIDGETS, Drag, searchCity(), T, Zone

### Community 115 - "UnpinPlan"
Cohesion: 0.50
Nodes (4): unpin_plan(), UnpinPlan, DeleteLocal, HideInDock

### Community 121 - "volume.rs"
Cohesion: 0.42
Nodes (7): clamp_level(), endpoint(), get_volume(), read(), set_volume(), Volume, write()

### Community 126 - "UpdateEffect"
Cohesion: 0.50
Nodes (4): UpdateEffect, Found, Lost, Nothing

### Community 129 - "initTheme"
Cohesion: 0.13
Nodes (20): Task 7: Window previews and count badge, initTheme(), CatalogApp, closeAddAppPanel(), closePreview(), getCatalog(), onPinnedChanged(), onPreviewUpdated() (+12 more)

### Community 131 - "Dock Features Implementation Plan"
Cohesion: 0.33
Nodes (5): Dock Features Implementation Plan, Global Constraints, Review Focus, Self-Review, DockMenuTarget

### Community 134 - "extract-notes.mjs"
Cohesion: 0.50
Nodes (3): end, lines, start

## Knowledge Gaps
- **535 isolated node(s):** `name`, `private`, `version`, `type`, `dev` (+530 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 949 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **39 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Dock (running apps, stable order)` connect `media.rs` to `windows_list.rs`, `Dock.tsx`?**
  _High betweenness centrality (0.058) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _535 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06666666666666667 - nodes in this community are weakly interconnected._
- **Why does `Visibility modes Fixed/Smart/Peek` connect `visibility.rs` to `Settings.tsx`?**
  _High betweenness centrality (0.049) - this node is a cross-community bridge._
- **Should `shelf.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11553030303030302 - nodes in this community are weakly interconnected._
- **Why does `Languages: English and Turkish` connect `super` to `useT`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **Should `DynamicIsland.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.0726775956284153 - nodes in this community are weakly interconnected._