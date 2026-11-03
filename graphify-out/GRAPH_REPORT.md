# Graph Report - MacDows  (2026-11-03)

## Corpus Check
- 85 files · ~60,479 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 22 file(s) not represented in the graph (top: .css 9, (none) 6, .lock 2)

## Summary
- 482 nodes · 784 edges · 22 communities (16 shown, 6 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 7 edges (avg confidence: 0.74)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `c4c8916d`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- windows_list.rs
- fullscreen.rs
- Notch.tsx
- media.rs
- package.json
- tauri.conf.json
- settings.rs
- compilerOptions
- lib.rs
- compilerOptions
- taskbar.rs
- default.json
- extract-notes.mjs
- macdows
- system_theme.rs
- visibility.rs
- MacDows Core (sub-project 0)

## God Nodes (most connected - your core abstractions)
1. `Notch()` - 16 edges
2. `compilerOptions` - 16 edges
3. `start()` - 13 edges
4. `collect()` - 12 edges
5. `apply()` - 12 edges
6. `useT()` - 11 edges
7. `WinBackend` - 8 edges
8. `current_status()` - 8 edges
9. `tray_window()` - 8 edges
10. `FileStore` - 7 edges

## Surprising Connections (you probably didn't know these)
- `visible_rects()` --references--> `Rect`  [EXTRACTED]
  src-tauri/src/windows_list.rs → src-tauri/src/fullscreen.rs
- `is_visible()` --calls--> `should_hide()`  [INFERRED]
  src-tauri/src/visibility.rs → src-tauri/src/fullscreen.rs
- `Notch()` --calls--> `usePomodoro()`  [EXTRACTED]
  src/notch/Notch.tsx → src/notch/Tools.tsx
- `PomodoroView()` --calls--> `useT()`  [EXTRACTED]
  src/notch/Tools.tsx → src/i18n.ts
- `Notch()` --calls--> `ToolsPanel()`  [EXTRACTED]
  src/notch/Notch.tsx → src/notch/Tools.tsx

## Import Cycles
- None detected.

## Communities (22 total, 6 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.06
Nodes (34): Store apps use window icon (WhatsApp fix), APP, AppWindow, BARS, class_of(), closed_windows_are_forgotten_and_reopened_go_to_the_end(), collect(), collect_rects() (+26 more)

### Community 1 - "fullscreen.rs"
Cohesion: 0.12
Nodes (10): APP, class_of(), covers_monitor(), LAST, MON, on_event(), probe(), Rect (+2 more)

### Community 2 - "Notch.tsx"
Cohesion: 0.05
Nodes (45): react, react-dom, @tauri-apps/api, AppWindow, focusWindow(), getMedia(), getSettings(), listWindows() (+37 more)

### Community 3 - "media.rs"
Cohesion: 0.09
Nodes (17): Event-driven, no polling for windows and media, APP, attach_current(), ATTACHED, get_media(), LAST, MANAGER, media_next() (+9 more)

### Community 4 - "package.json"
Cohesion: 0.06
Nodes (30): dependencies, framer-motion, react, react-dom, @tauri-apps/api, @tauri-apps/plugin-opener, devDependencies, @tauri-apps/cli (+22 more)

### Community 5 - "tauri.conf.json"
Cohesion: 0.08
Nodes (25): Tag-based signed release (GitHub Actions), app, security, windows, build, beforeBuildCommand, beforeDevCommand, devUrl (+17 more)

### Community 6 - "settings.rs"
Cohesion: 0.11
Nodes (12): Languages: English and Turkish, settings.json (corrupt file falls back to defaults), empty_string_gives_defaults(), load(), path(), save(), Settings, unknown_keys_are_ignored() (+4 more)

### Community 7 - "compilerOptions"
Cohesion: 0.11
Nodes (18): compilerOptions, allowImportingTsExtensions, isolatedModules, jsx, lib, module, moduleResolution, noEmit (+10 more)

### Community 8 - "lib.rs"
Cohesion: 0.22
Nodes (9): Single instance: second launch opens Settings, AppSettings, build_tray(), get_settings(), open_settings(), place_dock(), place_notch(), run() (+1 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "taskbar.rs"
Cohesion: 0.06
Nodes (32): ABS_AUTOHIDE, appbar_state(), battery_percent_from(), chevron_present_uia(), CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, current_status(), DETACHED_PROCESS (+24 more)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "extract-notes.mjs"
Cohesion: 0.40
Nodes (3): end, lines, start

### Community 17 - "system_theme.rs"
Cohesion: 0.08
Nodes (29): abgr(), ACCENT, apply(), Backup, backup_path(), broadcast(), DWM, ENABLED (+21 more)

### Community 18 - "visibility.rs"
Cohesion: 0.09
Nodes (18): Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, should_hide(), cursor(), foreground_exe(), inp(), Inputs, is_shell_menu() (+10 more)

### Community 21 - "MacDows Core (sub-project 0)"
Cohesion: 0.40
Nodes (6): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard)

## Knowledge Gaps
- **103 isolated node(s):** `Pomodoro`, `Dict`, `Mode`, `EVENT_OBJECT_CLOAKED`, `EVENT_OBJECT_UNCLOAKED` (+98 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 204 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **6 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Notch (top pill, media panel)` connect `MacDows Core (sub-project 0)` to `Notch.tsx`, `media.rs`?**
  _High betweenness centrality (0.103) - this node is a cross-community bridge._
- **What connects `Pomodoro`, `Dict`, `Mode` to the rest of the system?**
  _103 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06359189378057302 - nodes in this community are weakly interconnected._
- **Why does `Dock (running apps, stable order)` connect `MacDows Core (sub-project 0)` to `windows_list.rs`, `Notch.tsx`?**
  _High betweenness centrality (0.086) - this node is a cross-community bridge._
- **Should `fullscreen.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.11594202898550725 - nodes in this community are weakly interconnected._
- **Why does `react` connect `Notch.tsx` to `package.json`?**
  _High betweenness centrality (0.045) - this node is a cross-community bridge._
- **Should `Notch.tsx` be split into smaller, more focused modules?**
  _Cohesion score 0.05438184663536776 - nodes in this community are weakly interconnected._