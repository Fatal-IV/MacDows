# Handoff — where MacDows stands (2026-11-02)

Read this first when you continue on another machine.

## State
- Branch `v0.1-core` holds all work (plan `docs/superpowers/plans/2026-11-02-core-v0.1.md`, Tasks 1-8 done, Task 9 prepared). `main` only has the scaffold, spec and plan.
- 32 Rust unit tests pass (`cd src-tauri && cargo test`). `./node_modules/.bin/tsc --noEmit` is clean.
- `CHANGELOG.md` already has the detailed `[0.1.0]` notes; the release workflow uses them as the GitHub release body.
- **Not done yet:** merge `v0.1-core` into `main`, tag `v0.1.0`, first GitHub release. The release workflow (`.github/workflows/release.yml`) has never run, so expect to fix small CI issues on the first tag.
- A signed installer was built once locally with `bun run tauri build` (it needs the signing key in `TAURI_SIGNING_PRIVATE_KEY`).

## Run it
```
bun install
bun run tauri dev
```
Settings live in `%APPDATA%\MacDows\settings.json`. The tray icon is in the hidden-icons flyout (Tauri placeholder logo).

## Release secrets (important)
- GitHub Actions secret `TAURI_SIGNING_PRIVATE_KEY` is already set on `Fatal-IV/MacDows`.
- The private key file was created on the work PC at `~/.macdows-keys/updater.key` (empty password). It is NOT in the repo. Back it up: if it is lost, signed updates cannot be produced. Public key is in `src-tauri/tauri.conf.json`.

## Decisions to keep
- Licensed GPL-3.0-only (changed from MIT on 2026-11-03). The notch behavior and styling are adapted from Nectar (kua8, GPL-3.0), so the whole project had to move to GPL-3.0; keep the attribution in `NOTICE`. Do not copy Nectar's SF Symbols SVGs (Apple license); MacDows draws its own icons.
- Commits show only `Fatal-IV`. Use the repo-local identity `Fatal-IV <72112242+Fatal-IV@users.noreply.github.com>`; no co-author or generated-by lines.
- Default visibility is Fixed: bars always visible, hidden only for fullscreen apps. Smart and Peek are opt-in.
- The Windows taskbar is untouched in v0.1. Hiding or replacing it is v0.2 and needs a guard process that restores it.
- Event-driven where possible; the visibility loop samples the cursor every 100 ms (250 ms when both bars are Fixed).

## Known gaps (deferred)
- No in-app update check or install yet (updater plugin is only registered).
- `csp` is null in `tauri.conf.json`; README is a stub; tray icon is a placeholder; tray menu language changes after a restart; Smart mode can fully hide the notch; primary monitor only.

## Roadmap
1. Release v0.1.0.
2. v0.2: taskbar hiding with guard process, in-app update check.
3. System monitor, clipboard history, launcher / Start replacement, notification center. A spike showed `UserNotificationListener` can read toasts from a plain unpackaged exe.

## Knowledge graph
`graphify-out/` has `graph.html` (open in a browser), `graph.json` and `GRAPH_REPORT.md`, built from the code plus the spec/plan/changelog concepts.
