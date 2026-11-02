# MacDows In-App Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** MacDows checks GitHub for a newer signed release, shows a quiet notch dot and a Settings card, and installs only when the user presses **Update and restart**; the work is included in the v0.1.0 notes.

**Architecture:** One Rust module `updates.rs` holds pure decisions (tested) and the effects (background task, status state, three commands, one event) on top of `tauri-plugin-updater`. Settings gets two fields. The UI only renders `UpdateStatus` and calls the commands.

**Tech Stack:** Tauri 2 + `tauri-plugin-updater` 2.14, Rust, React + TypeScript, Bun.

**Spec:** `docs/superpowers/specs/2026-11-02-update-check-design.md` (read it with this plan).

## Global Constraints
- Check 30 s after launch (`FIRST_CHECK_DELAY_SECS = 30`), then every 12 h (`CHECK_INTERVAL_SECS = 12 * 3600`); skip when the last check is less than 12 h old.
- `check_updates` default `true`; `last_update_check: Option<u64>` unix seconds default `None`; old `settings.json` files must still load.
- Nothing is downloaded or installed before the user presses **Update and restart**.
- Release notes shown in the UI are cut to `NOTES_MAX_CHARS = 600` characters (including the final `…`).
- The endpoint and public key stay as in `src-tauri/tauri.conf.json`; `MACDOWS_UPDATE_ENDPOINT` is honored in debug builds only.
- Strings in English and Turkish; blue dot `#4cc2ff`, 6 px, top right of the collapsed notch pill, only for `available`.
- Commits authored only as `Fatal-IV`, no co-author or generated-by lines. Branch `feat/update-check`.
- Checks before each commit: `cd src-tauri && cargo test`, `./node_modules/.bin/tsc --noEmit`, `node --test scripts/`.

## Review Focus
- Release notes with multi-byte or very long text → cut on a character boundary, no panic (Task 1 test).
- Clock moved back or `last_update_check` in the future → the check still runs (Task 1 test).
- `settings.json` from v0.1.0 without the new fields → loads with defaults (Task 1 test).
- "Check now" or install pressed while a check/install is already running → ignored, never a double install (Task 1 test).
- Offline or failing background check → no error shown to the user; a manual check shows one sentence (Task 1 test, Task 5 check).

## File Structure
- `src-tauri/src/updates.rs` — pure logic + effects. `src-tauri/src/settings.rs` — two new fields. `src-tauri/src/lib.rs` — register commands and start the task.
- `src/updates.ts`, `src/notch/Notch.tsx`, `src/settings/Settings.tsx`, `src/locales/{en,tr}.json`.
- `scripts/fake-update-server.mjs` — local `latest.json` server for the end-to-end test.

---

### Task 1: Pure update logic and settings fields (tests first)
**Files:** Create `src-tauri/src/updates.rs` (tests inside); Modify `src-tauri/src/settings.rs`, `src-tauri/src/lib.rs` (`mod updates;`).
**Interfaces:** Produces:
- `enum UpdateStatus { Idle, Checking, UpToDate { checked_at: u64 }, Available { version: String, notes: String }, Installing { percent: Option<u8> }, Error { message: String, during: String } }` (`Serialize`, `Clone`, `PartialEq`, `#[serde(tag = "kind", rename_all = "snake_case")]`)
- `enum CheckOutcome { UpToDate, Available { version: String, notes: String }, Failed(String) }`
- `fn should_check(last: Option<u64>, now: u64, interval_secs: u64) -> bool`
- `fn shorten_notes(notes: &str, max_chars: usize) -> String`
- `fn can_start(status: &UpdateStatus) -> bool` (false for `Checking` and `Installing`)
- `fn after_check(prev: &UpdateStatus, outcome: CheckOutcome, user_initiated: bool, now: u64) -> UpdateStatus` (a `Failed` outcome gives `Error { during: "check" }` only when `user_initiated`, otherwise returns `prev.clone()`)
- constants from Global Constraints; `Settings.check_updates: bool`, `Settings.last_update_check: Option<u64>`.
- [ ] **Step 1: Write the failing tests:** `should_check`: `None`→true, last = now−60 → false, last = now−interval → true, last = now−interval−1 → true, last = now+500 → true. `shorten_notes`: `"hi"`→`"hi"`; text of exactly 600 chars unchanged; 700 `a`s → 600 chars ending in `…`; `"ğüşiöçİ".repeat(200)` → at most 600 chars, valid UTF‑8, ends in `…`; `"  x  "`→`"x"`; `""`→`""`. `can_start`: Idle/UpToDate/Available/Error → true; Checking/Installing → false. `after_check`: Available outcome → `Available`; UpToDate → `UpToDate { checked_at: now }`; Failed + user → `Error { during: "check" }`; Failed + background with prev `UpToDate` → equals prev. Settings: defaults are `true`/`None`; `Settings::from_json(r#"{"language":"tr"}"#)` keeps `check_updates == true`.
- [ ] **Step 2:** `cd src-tauri && cargo test updates settings` → Expected: FAIL (items missing).
- [ ] **Step 3:** Implement the items above; `shorten_notes` counts `chars()` and cuts at `max_chars - 1` characters before adding `…`.
- [ ] **Step 4:** `cargo test` → PASS. Commit `feat: update decisions and settings fields`.

### Task 2: Prove the update path with a local fake release
**Files:** Create `scripts/fake-update-server.mjs`, `docs/superpowers/notes/2026-11-02-update-facts.md`.
**Interfaces:** Produces `node scripts/fake-update-server.mjs <dir> <version> <port>` serving `<dir>/latest.json` (fields `version`, `notes`, `pub_date`, `platforms."windows-x86_64".{signature,url}`) and the package; and the recorded answers to: does the updater accept `http://localhost`, does the NSIS update relaunch MacDows, what exact installed version the app reports.
- [ ] **Step 1:** Build a signed fake `0.1.1` package: `TAURI_SIGNING_PRIVATE_KEY="$HOME/.macdows-keys/updater.key" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" bun run tauri build --config '{"version":"0.1.1"}'`. Expected: `MacDows_0.1.1_x64-setup.exe` and `.sig` exist. Restore the source files untouched afterwards (`git status` clean).
- [ ] **Step 2:** Write the server script (static `latest.json` built from the `.sig` text, `Content-Type: application/json`, package served with `Content-Length`).
- [ ] **Step 3:** Record in the notes file whether an `http://localhost` endpoint is accepted by the plugin in a debug build; if it is refused, record the `dangerousInsecureTransportProtocol` override file name to use only for `tauri dev` (`src-tauri/tauri.dev-update.conf.json`, never bundled).
- [ ] **Step 4:** Commit `test: fake release server and update facts`.

### Task 3: Update effects (state, background task, commands)
**Files:** Modify `src-tauri/src/updates.rs`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json` if the commands need it.
**Interfaces:** Consumes Task 1 items. Produces commands `get_update_status() -> UpdateStatus`, `check_for_update() -> UpdateStatus`, `install_update() -> Result<(), String>`, the event `update-status` (payload `UpdateStatus`), and `fn start(app: AppHandle)` (called from `setup`). Persists `last_update_check` through the existing settings save.
- [ ] **Step 1: Write the failing test** `status_state_blocks_a_second_install`: with the shared state set to `Installing`, `begin_install(&state)` returns `false` and leaves the status unchanged; from `Available` it returns `true` and sets `Installing`.
- [ ] **Step 2:** `cargo test updates` → FAIL. **Step 3:** Implement the state holder and `begin_install`, then the commands using `tauri_plugin_updater::UpdaterExt` (`check`, `download_and_install` with a progress callback emitting `installing`), the debug-only endpoint override, and the background loop (first check after `FIRST_CHECK_DELAY_SECS`, then each `CHECK_INTERVAL_SECS`, honoring `check_updates` and `should_check`; failures only logged to `%APPDATA%\MacDows\logs`).
- [ ] **Step 4:** `cargo test` → PASS; `cargo check` clean. Commit `feat: update check and install commands`.

### Task 4: UI
**Files:** Create `src/updates.ts`; Modify `src/notch/Notch.tsx`, `src/notch/Notch.css`, `src/settings/Settings.tsx`, `src/settings/Settings.css`, `src/locales/en.json`, `src/locales/tr.json`.
**Interfaces:** Consumes the commands and event from Task 3. `src/updates.ts` exports `type UpdateStatus` (same shape as Task 1), `getUpdateStatus()`, `checkForUpdate()`, `installUpdate()`, `useUpdateStatus(): UpdateStatus`.
- [ ] **Step 1:** Add the typed bridge and the hook (initial `getUpdateStatus()`, then `update-status` events).
- [ ] **Step 2:** Notch: 6 px `#4cc2ff` dot at the top right of the collapsed pill when `kind === "available"`. Settings: "Updates" card per the spec (version, status line, **Check now**, **Update and restart** only for `available`, notes, automatic-check switch via `setSetting("check_updates", …)`), strings in both locales.
- [ ] **Step 3:** `./node_modules/.bin/tsc --noEmit` → clean. Commit `feat: update UI`.

### Task 5: End-to-end on this PC
**Files:** evidence appended to `docs/superpowers/notes/2026-11-02-update-facts.md`.
- [ ] **Step 1:** Ask the user's permission, then: start the fake server (Task 2) for `0.1.1`, start a debug MacDows at `0.1.0` with `MACDOWS_UPDATE_ENDPOINT`. Expected: the dot appears within the first check, the card shows `0.1.1` and the notes (screenshots).
- [ ] **Step 2:** Press **Update and restart** (UI Automation, no mouse movement). Expected: MacDows ends, installs `0.1.1`, runs again, and `DisplayVersion` is `0.1.1`. If the app does not come back by itself, add the relaunch step to Task 3 and repeat. Uninstall afterwards.
- [ ] **Step 3:** Tamper with the served package (one byte) and repeat. Expected: not installed, one-sentence error, installed version unchanged. Stop the server and press **Check now**: one-sentence error; a background check with the server down shows nothing.
- [ ] **Step 4:** Record results, including anything not verified (a real GitHub release, proxies). Commit `docs: update verification notes`.

### Task 6: Docs and the v0.1.0 release notes
**Files:** Modify `CHANGELOG.md`, `README.md`.
- [ ] **Step 1:** Move the logo, branded installer and release-asset entries from `[Unreleased]` into `[0.1.0]` under **Added**; add the update entry (check every 12 h, notch dot, Settings card, installs only on approval, can be switched off); delete the line "No in-app update check yet…" from Known limitations and the "The app does not check for updates yet" sentence in the signed-releases bullet.
- [ ] **Step 2:** README: short "Updates" paragraph. Run `node scripts/extract-notes.mjs 0.1.0` and confirm the notes contain the update, installer and logo entries.
- [ ] **Step 3:** `cargo test`, `tsc`, `node --test scripts/` → all green. Commit `docs: v0.1.0 notes include logo, installer and updates`.

---
## Self-review
- **Spec coverage:** behavior and intervals → Tasks 1, 3; status model → Task 1; units 1–4 → Tasks 1, 3, 4; errors → Tasks 3, 5; testing list → Tasks 1, 2, 5; relaunch question → Tasks 2, 5; v0.1.0 inclusion → Task 6; out-of-scope respected. Merging and tagging stay outside this plan.
- **Type consistency:** `UpdateStatus`, `CheckOutcome`, `should_check`, `shorten_notes`, `can_start`, `after_check`, `begin_install`, command and event names are identical across Tasks 1, 3, 4.
- **Not verifiable here:** a real GitHub release, proxy setups; stated in Task 5.
