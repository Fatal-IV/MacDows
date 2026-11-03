# Graph Report - MacDows  (2026-11-03)

## Corpus Check
- 101 files · ~72,431 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 24 file(s) not represented in the graph (top: .css 9, (none) 6, .ico 3)

## Summary
- 1496 nodes · 2563 edges · 74 communities (60 shown, 14 thin omitted)
- Extraction: 98% EXTRACTED · 2% INFERRED · 0% AMBIGUOUS · INFERRED: 43 edges (avg confidence: 0.88)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cdc54d8d`
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
- taskbar.rs
- default.json
- extract-notes.mjs
- macdows
- system_theme.rs
- visibility.rs
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
- bluetooth.rs
- logic.rs
- useT
- hide
- Window
- compilerOptions
- TaskbarState
- installer/src-tauri/tauri.conf.json
- MacDows Installer and Logo — Design
- make-icons.mjs
- quick_settings.rs
- rig
- File Structure
- MacDows Taskbar Takeover — Design
- build-installer.mjs
- Handoff — where MacDows stands (2026-11-02)
- Dinamik Ada ayarları: tasarım
- run_install
- make-icons.test.mjs
- secure_store.rs
- fake-update-server.mjs
- Installer facts verified on a real PC (2026-11-02)
- In-app update facts (2026-11-02)
- MacDows In-App Update — Design
- SetupError
- i18n.rs
- installer/src-tauri/capabilities/default.json
- notifications.rs
- Taskbar takeover facts verified on a real PC (2026-11-02, Windows 11, one monitor)
- Stage
- super
- MacDows
- MacDows.Setup.csproj
- Shelf.tsx
- macdows-setup
- bridge.ts
- install_state
- kind_of
- ShellAction
- Backend

## God Nodes (most connected - your core abstractions)
1. `Notch()` - 34 edges
2. `useT()` - 28 edges
3. `TaskbarState` - 25 edges
4. `Window` - 18 edges
5. `UpdateStatus` - 17 edges
6. `compilerOptions` - 16 edges
7. `compilerOptions` - 16 edges
8. `Notice` - 15 edges
9. `hide()` - 15 edges
10. `run_install()` - 14 edges

## Surprising Connections (you probably didn't know these)
- `Task 6: UI and uninstall hook` --references--> `useSystemStatus()`  [INFERRED]
  docs/superpowers/plans/2026-11-02-taskbar-takeover.md → src/system.ts
- `4. Arayüz` --references--> `Row()`  [INFERRED]
  docs/superpowers/specs/2026-11-03-dynamic-island-settings-design.md → src/ui/controls.tsx
- `State` --references--> `main()`  [INFERRED]
  docs/HANDOFF.md → scripts/build-installer.mjs
- `Self-review` --references--> `shellAction`  [INFERRED]
  docs/superpowers/plans/2026-11-02-taskbar-takeover.md → src/system.ts
- `Self-review` --references--> `UpdateStatus`  [INFERRED]
  docs/superpowers/plans/2026-11-02-update-check.md → src/updates.ts

## Import Cycles
- None detected.

## Communities (74 total, 14 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.06
Nodes (35): Store apps use window icon (WhatsApp fix), APP, AppWindow, BARS, class_of(), closed_windows_are_forgotten_and_reopened_go_to_the_end(), collect(), collect_rects() (+27 more)

### Community 1 - "shelf.rs"
Cohesion: 0.06
Nodes (40): BOOTSTRAPPER_URL, CLIENT_KEY, CREATE_NO_WINDOW, download(), download_args(), download_command_follows_redirects_and_names_the_target(), ensure_webview2(), first_pv() (+32 more)

### Community 2 - "DynamicIsland.tsx"
Cohesion: 0.10
Nodes (34): Props, silhouette(), SurfaceEdge, SurfaceShape(), addToShelf(), clearShelf(), getMedia(), getShelf() (+26 more)

### Community 3 - "media.rs"
Cohesion: 0.06
Nodes (28): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard), APP, attach_current() (+20 more)

### Community 4 - "package.json"
Cohesion: 0.05
Nodes (38): dependencies, @crabnebula/tauri-plugin-drag, framer-motion, react, react-dom, @tauri-apps/api, @tauri-apps/plugin-opener, devDependencies (+30 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.07
Nodes (29): Tag-based signed release (GitHub Actions), app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+21 more)

### Community 6 - "settings.rs"
Cohesion: 0.16
Nodes (14): settings.json (corrupt file falls back to defaults), a_v010_settings_file_without_update_fields_still_loads(), empty_string_gives_defaults(), load(), path(), save(), Settings, taskbar_fields_default_to_replace_notice_pending_and_autostart() (+6 more)

### Community 7 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+10 more)

### Community 8 - "src-tauri/src/lib.rs"
Cohesion: 0.12
Nodes (24): Single instance: second launch opens Settings, activate_tray_item(), apply_replace_taskbar(), AppSettings, bring_to_front(), build_tray(), close_tray(), close_tray_panel() (+16 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "taskbar.rs"
Cohesion: 0.07
Nodes (33): ABS_AUTOHIDE, appbar_state(), battery_percent_from(), chevron_present_uia(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, current_status(), DETACHED_PROCESS (+25 more)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "extract-notes.mjs"
Cohesion: 0.40
Nodes (3): end, lines, start

### Community 17 - "system_theme.rs"
Cohesion: 0.08
Nodes (30): accent_from_registry(), get_system_accent_color(), abgr(), ACCENT, apply(), Backup, backup_path(), broadcast() (+22 more)

### Community 18 - "visibility.rs"
Cohesion: 0.05
Nodes (31): Event-driven, no polling for windows and media, Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, APP, class_of(), covers_monitor(), LAST, MON (+23 more)

### Community 19 - "updates.rs"
Cohesion: 0.07
Nodes (38): after_check(), available(), begin_check(), begin_install(), build_updater(), can_start(), check_for_update(), CHECK_INTERVAL_SECS (+30 more)

### Community 20 - "visualizer.rs"
Cohesion: 0.07
Nodes (31): a_louder_tone_shows_a_higher_bar(), a_steady_tone_lights_its_own_band(), analyze(), BANDS, bars_rise_at_once_and_fall_gradually(), capture_session(), CEIL_DB, EDGES (+23 more)

### Community 21 - "api.ts"
Cohesion: 0.08
Nodes (35): File Structure, Global Constraints, MacDows Installer and Logo Implementation Plan, Review Focus, Self-review, Task 1: Verify the real Windows facts before coding against them, Task 2: Logo sources and icon generation, Task 3: Pure installer logic (Rust, tests first) (+27 more)

### Community 22 - "tray_panel.rs"
Cohesion: 0.12
Nodes (25): activate(), automation(), BUSY, capture(), class_name(), close_flyout(), collect(), find_by_class() (+17 more)

### Community 23 - "Dock.tsx"
Cohesion: 0.13
Nodes (18): initTheme(), AppWindow, focusWindow(), listWindows(), onWindowsChanged(), BELL, CHEVRON, DESKTOP (+10 more)

### Community 24 - "install.rs"
Cohesion: 0.09
Nodes (20): apply_autostart_choice(), BUNDLED_VERSION, CREATE_NO_WINDOW, ErrorEvent, extract_payload(), extract_payload_works_with_non_ascii_dir(), InstalledInfo, is_stub_len() (+12 more)

### Community 25 - "Settings.tsx"
Cohesion: 0.11
Nodes (24): framer-motion, setSetting(), Settings, Visibility, ACCENTS, Appearance(), MODES, useStored() (+16 more)

### Community 26 - "theme.ts"
Cohesion: 0.13
Nodes (24): applyBackdrop(), clearBackdrop(), enableBackdrop(), STYLE_BLUR, usesBackdrop(), VARS, WallpaperBackdrop, wallpaperRect() (+16 more)

### Community 27 - "taskbar_plan.rs"
Cohesion: 0.09
Nodes (12): a_restart_after_a_crash_keeps_the_stored_original(), hidden(), hidden_icons_need_the_chevron_and_a_hidden_taskbar(), HidePlan, hiding_records_the_real_original(), plan_hide(), plan_restore(), restore_puts_back_each_original() (+4 more)

### Community 28 - "installer/package.json"
Cohesion: 0.07
Nodes (28): dependencies, react, react-dom, @tauri-apps/api, devDependencies, @tauri-apps/cli, @types/react, @types/react-dom (+20 more)

### Community 29 - "bluetooth.rs"
Cohesion: 0.13
Nodes (9): address_hex(), batteries(), BATTERY_KEY, BtDevice, get_bluetooth_devices(), radio(), read_devices(), set_bluetooth_connection() (+1 more)

### Community 30 - "logic.rs"
Cohesion: 0.11
Nodes (13): decide(), Decision, Fresh, Newer, Older, Reinstall, exit_code_message(), exit_code_zero_is_a_caller_bug() (+5 more)

### Community 31 - "useT"
Cohesion: 0.12
Nodes (24): clearNotificationHistory(), getNotificationHistory(), getSettings(), Notice, onNotificationHistoryChanged(), onSettingsChanged(), Dict, dicts (+16 more)

### Community 32 - "hide"
Cohesion: 0.16
Nodes (12): hide(), hide_then_restore_returns_to_auto_hide_off(), hide_then_restore_returns_to_auto_hide_on_when_it_was_on(), HideError, StateNotSaved, hiding_again_after_a_crash_keeps_the_first_original(), MemStore, MovesTheWindowAway (+4 more)

### Community 33 - "Window"
Cohesion: 0.07
Nodes (24): MacDows.Setup, Application, App, Bar, Box, DonePanel, DoneText, DoneTitle (+16 more)

### Community 34 - "compilerOptions"
Cohesion: 0.11
Nodes (17): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+9 more)

### Community 35 - "TaskbarState"
Cohesion: 0.14
Nodes (9): guard_should_restore(), needs_final_show(), needs_stale_restore(), state_belongs_to(), state_from_json(), state_to_json(), Store, TaskbarState (+1 more)

### Community 36 - "installer/src-tauri/tauri.conf.json"
Cohesion: 0.12
Nodes (16): app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl, frontendDist (+8 more)

### Community 37 - "MacDows Installer and Logo — Design"
Cohesion: 0.13
Nodes (14): Architecture, Build and release, Decisions (user), Errors, Goal, Install behavior, Logo, MacDows Installer and Logo — Design (+6 more)

### Community 38 - "make-icons.mjs"
Cohesion: 0.14
Nodes (13): png-to-ico, @resvg/resvg-js, appIcons, assets, icoSizes, installerIcons, main, pngFor() (+5 more)

### Community 39 - "quick_settings.rs"
Cohesion: 0.07
Nodes (28): airplane_on(), airplane_tile(), banners_on(), clamp_percent(), CREATE_NO_WINDOW, get_brightness(), get_quick_settings(), has_battery() (+20 more)

### Community 40 - "rig"
Cohesion: 0.20
Nodes (4): FakeBackend, FakeStore, nothing_is_hidden_when_the_state_cannot_be_saved(), rig()

### Community 41 - "File Structure"
Cohesion: 0.14
Nodes (13): File Structure, Global Constraints, MacDows Taskbar Takeover Implementation Plan, Review Focus, Self-review, Task 1: Pure plans and key steps (tests first), Task 2: Hide and restore over traits (tests first), Task 3: Real Windows backend, state file and shell actions (+5 more)

### Community 42 - "MacDows Taskbar Takeover — Design"
Cohesion: 0.15
Nodes (12): Behavior details, Decisions (user), Errors, Goal, How the taskbar is hidden, MacDows Taskbar Takeover — Design, Risks, Scope (+4 more)

### Community 43 - "build-installer.mjs"
Cohesion: 0.24
Nodes (7): State, Task 8: Build script, release workflow, docs, findPayload(), main(), root, run(), sumsLine()

### Community 44 - "Handoff — where MacDows stands (2026-11-02)"
Cohesion: 0.22
Nodes (8): Decisions to keep, Handoff — where MacDows stands (2026-11-02), Knowledge graph, Known gaps (deferred), Release secrets (important), Roadmap, Run it, Start and Search (2026-11-03)

### Community 45 - "Dinamik Ada ayarları: tasarım"
Cohesion: 0.17
Nodes (11): 1. Amaç, 2.1 Ekranlar, 2.2 Gezinme, 2.3 Görünürlük ve yerleşim, 2. Ayar modeli, 3. Davranış kuralları, 4. Arayüz, 5. Aşamalar (+3 more)

### Community 46 - "run_install"
Cohesion: 0.47
Nodes (6): emit_stage(), fail(), run_install(), start_install(), write_log(), stage_percent()

### Community 48 - "secure_store.rs"
Cohesion: 0.26
Nodes (8): blob(), MAGIC, open(), plain_files_and_damaged_data_are_not_opened(), seal(), take(), the_sealed_bytes_do_not_contain_the_text(), what_is_sealed_can_be_opened_again()

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
Cohesion: 0.08
Nodes (28): File Structure, Global Constraints, MacDows In-App Update Implementation Plan, Review Focus, Self-review, Task 1: Pure update logic and settings fields (tests first), Task 2: Prove the update path with a local fake release, Task 3: Update effects (state, background task, commands) (+20 more)

### Community 53 - "SetupError"
Cohesion: 0.33
Nodes (7): io(), launch_app(), run_nsis(), SetupError, Io, NotInstalled, PayloadMissing

### Community 54 - "i18n.rs"
Cohesion: 0.22
Nodes (3): Languages: English and Turkish, taskbar_notice(), the_first_run_notice_explains_how_to_undo_it()

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

### Community 59 - "super"
Cohesion: 0.25
Nodes (5): Cli, Guard, Normal, RestoreTaskbar, parse()

### Community 60 - "MacDows"
Cohesion: 0.40
Nodes (4): Develop, Install, MacDows, Updates

### Community 62 - "Shelf.tsx"
Cohesion: 0.43
Nodes (6): @crabnebula/tauri-plugin-drag, openShelfItem(), revealShelfItem(), ShelfItem, ShelfScreen(), Tile()

### Community 69 - "bridge.ts"
Cohesion: 0.08
Nodes (35): BtDevice, BtKind, getBluetoothDevices(), getBrightness(), getQuickSettings(), getVolume(), MediaInfo, mediaNext() (+27 more)

### Community 70 - "install_state"
Cohesion: 0.50
Nodes (4): install_state(), InstallInfo, log_path(), system_locale()

### Community 73 - "ShellAction"
Cohesion: 0.17
Nodes (12): hidden_icons_jump_to_the_chevron_with_home_before_pressing_enter(), key_steps(), KeyStep, Chord, Wait, ShellAction, HiddenIcons, Notifications (+4 more)

## Knowledge Gaps
- **395 isolated node(s):** `net8.0-windows`, `Microsoft.NET.Sdk`, `ProgressBar`, `Button`, `InstalledExe` (+390 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 655 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **14 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Visibility modes Fixed/Smart/Peek` connect `visibility.rs` to `Settings.tsx`?**
  _High betweenness centrality (0.089) - this node is a cross-community bridge._
- **What connects `net8.0-windows`, `Microsoft.NET.Sdk`, `ProgressBar` to the rest of the system?**
  _395 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06038961038961039 - nodes in this community are weakly interconnected._
- **Why does `Dock (running apps, stable order)` connect `media.rs` to `windows_list.rs`, `Dock.tsx`?**
  _High betweenness centrality (0.088) - this node is a cross-community bridge._
- **Should `shelf.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05683060109289618 - nodes in this community are weakly interconnected._
- **Why does `framer-motion` connect `Settings.tsx` to `DynamicIsland.tsx`, `package.json`, `Dock.tsx`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Should `DynamicIsland.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09716599190283401 - nodes in this community are weakly interconnected._