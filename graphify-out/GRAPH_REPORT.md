# Graph Report - MacDows  (2026-11-02)

## Corpus Check
- Corpus is ~9,016 words - fits in a single context window. You may not need a graph.

## Summary
- 337 nodes · 509 edges · 17 communities (13 shown, 4 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 6 edges (avg confidence: 0.72)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- windows_list.rs
- fullscreen.rs
- bridge.ts
- media.rs
- package.json
- tauri.conf.json
- settings.rs
- compilerOptions
- lib.rs
- compilerOptions
- MacDows Core (sub-project 0)
- default.json
- extract-notes.mjs
- macdows

## God Nodes (most connected - your core abstractions)
1. `compilerOptions` - 16 edges
2. `collect()` - 12 edges
3. `Rect` - 10 edges
4. `start()` - 10 edges
5. `useT()` - 9 edges
6. `AppSettings` - 6 edges
7. `set_setting()` - 6 edges
8. `run()` - 6 edges
9. `Settings` - 6 edges
10. `load()` - 6 edges

## Surprising Connections (you probably didn't know these)
- `visible_rects()` --references--> `Rect`  [EXTRACTED]
  src-tauri/src/windows_list.rs → src-tauri/src/fullscreen.rs
- `is_visible()` --calls--> `should_hide()`  [INFERRED]
  src-tauri/src/visibility.rs → src-tauri/src/fullscreen.rs
- `overlap_probe()` --references--> `Rect`  [EXTRACTED]
  src-tauri/src/visibility.rs → src-tauri/src/fullscreen.rs
- `point_in_rect()` --references--> `Rect`  [EXTRACTED]
  src-tauri/src/visibility.rs → src-tauri/src/fullscreen.rs
- `rect_intersects()` --references--> `Rect`  [EXTRACTED]
  src-tauri/src/visibility.rs → src-tauri/src/fullscreen.rs

## Import Cycles
- None detected.

## Communities (17 total, 4 thin omitted)

### Community 0 - "windows_list.rs"
Cohesion: 0.05
Nodes (35): Store apps use window icon (WhatsApp fix), APP, AppWindow, BARS, class_of(), closed_windows_are_forgotten_and_reopened_go_to_the_end(), collect(), collect_rects() (+27 more)

### Community 1 - "fullscreen.rs"
Cohesion: 0.06
Nodes (23): Rule: hide bars only for fullscreen apps (default Fixed), Visibility modes Fixed/Smart/Peek, APP, class_of(), covers_monitor(), LAST, MON, on_event() (+15 more)

### Community 2 - "bridge.ts"
Cohesion: 0.11
Nodes (31): react, react-dom, @tauri-apps/api, AppWindow, focusWindow(), getMedia(), getSettings(), listWindows() (+23 more)

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
Cohesion: 0.24
Nodes (9): Single instance: second launch opens Settings, AppSettings, build_tray(), get_settings(), open_settings(), place_dock(), place_notch(), run() (+1 more)

### Community 9 - "compilerOptions"
Cohesion: 0.25
Nodes (7): compilerOptions, allowSyntheticDefaultImports, composite, module, moduleResolution, skipLibCheck, include

### Community 10 - "MacDows Core (sub-project 0)"
Cohesion: 0.40
Nodes (6): Dock (running apps, stable order), Decision: written from scratch, MIT, no Nectar code copied, MacDows Core (sub-project 0), Notch (top pill, media panel), Roadmap: system monitor, clipboard, launcher/Start replacement, notifications, Taskbar strategy: staged (v0.1 untouched, v0.2 hide + guard)

### Community 11 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 12 - "extract-notes.mjs"
Cohesion: 0.40
Nodes (3): end, lines, start

## Knowledge Gaps
- **94 isolated node(s):** `name`, `private`, `version`, `type`, `dev` (+89 more)
  These have ≤1 connection - possible missing edges or undocumented components. (Counts symbols only; 158 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **4 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Dock (running apps, stable order)` connect `MacDows Core (sub-project 0)` to `windows_list.rs`, `bridge.ts`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **What connects `name`, `private`, `version` to the rest of the system?**
  _94 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `windows_list.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.05480225988700565 - nodes in this community are weakly interconnected._
- **Why does `Notch (top pill, media panel)` connect `MacDows Core (sub-project 0)` to `bridge.ts`, `media.rs`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Should `fullscreen.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.062409288824383166 - nodes in this community are weakly interconnected._
- **Why does `react` connect `bridge.ts` to `package.json`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Should `bridge.ts` be split into smaller, more focused modules?**
  _Cohesion score 0.1076923076923077 - nodes in this community are weakly interconnected._