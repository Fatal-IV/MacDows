# MacDows In-App Update — Design

Date: 2026-11-02 · Status: awaiting user review
Parent specs: `2026-11-02-core-design.md`, `2026-11-02-installer-design.md`. This spec adds the update check and install that the v0.1.0 release notes list as missing. It does not change the installer or the release workflow.

## Goal
MacDows finds a newer GitHub release by itself, tells the user quietly, and installs it only when the user asks. Releases are already built and signed by CI; `latest.json` is published next to each release (see the installer spec).

## Success criteria
- A newer release shows a small blue dot on the notch and an update card in Settings, without any dialog.
- Nothing is downloaded or installed until the user presses **Update and restart**.
- After the update MacDows is running again at the new version.
- No network, no release, or a bad signature never breaks the app: the background check is silent, a manual check explains the problem in one sentence.
- The user can turn the automatic check off.

## Decisions (user)
- Behavior: *notify, install on approval* (not fully automatic, not manual-only).

## Behavior
- First check 30 seconds after launch, then every 12 hours while running. A check is skipped when the last one was less than 12 hours ago (so restarting does not hit GitHub again).
- Only when `check_updates` is on (default on). The **Check now** button always works.
- The update comes from the endpoint already in `tauri.conf.json` (`https://github.com/Fatal-IV/MacDows/releases/latest/download/latest.json`) and is verified with the public key in the same file; an update with a bad signature is never installed.
- Debug builds only: the environment variable `MACDOWS_UPDATE_ENDPOINT` replaces the endpoint (used to test against a local server). Release builds ignore it.

## Status model
`UpdateStatus` (sent to the UI as JSON, field `kind`):
- `idle` — never checked in this run.
- `checking`
- `up_to_date` (with `checked_at`)
- `available` (`version`, `notes`)
- `installing` (`percent` 0–100, or indeterminate when the size is unknown)
- `error` (`message`, one sentence, `during`: `"check"` or `"install"`)

A background error is stored in the status as `error` but only shown in Settings when the user started the check; the notch never shows a dot for it.

## Units
1. **`src-tauri/src/updates.rs` — pure logic (unit tested)**
   - `fn should_check(last: Option<u64>, now: u64, interval_secs: u64) -> bool`: `None` → true; `now < last` (clock moved back) → true; otherwise `now - last >= interval_secs`.
   - `fn shorten_notes(notes: &str, max_chars: usize) -> String`: trims, and when longer than `max_chars` characters cuts on a character boundary and appends `…` (the result is at most `max_chars` characters including the `…`). Must not panic on multi-byte text such as Turkish.
   - `const CHECK_INTERVAL_SECS: u64 = 12 * 3600`, `const FIRST_CHECK_DELAY_SECS: u64 = 30`, `const NOTES_MAX_CHARS: usize = 600`.
2. **`updates.rs` — effects:** background task (started from `setup`), the shared `UpdateStatus` state, and the commands `get_update_status() -> UpdateStatus`, `check_for_update() -> UpdateStatus`, `install_update()` (downloads, emits `update-status` with `installing` progress, installs; the NSIS package ends and restarts the app). Every state change emits the event `update-status`.
3. **Settings:** two new fields in `settings.rs`: `check_updates: bool` (default `true`) and `last_update_check: Option<u64>` (unix seconds, default `None`). Existing tests keep passing because missing fields fall back to defaults.
4. **UI:** `src/updates.ts` (typed commands/events, `useUpdateStatus()`), a blue dot in `Notch.tsx` (6 px, `#4cc2ff`, top right of the collapsed pill, only for `available`), and an "Updates" card in `Settings.tsx`: installed version, status line, **Check now**, **Update and restart** (only for `available`), the release notes text, and the **Check for updates automatically** switch. Strings in `en.json` and `tr.json`.

## Errors
One sentence each, no stack traces: *Could not check for updates (no connection?)*, *Update download failed*, *The update could not be verified and was not installed*, *Installing the update failed*. Details go to the log (`%APPDATA%\MacDows\logs`).

## Testing
- Unit tests (written first): `should_check` (None, recent, exactly at interval, past interval, clock moved back); `shorten_notes` (short text unchanged, exact limit unchanged, long text cut and ends with `…`, Turkish text such as `ğüşiöçİ` repeated is cut on a character boundary, empty text); settings defaults for the two new fields; a settings file without them still loads.
- Type check: `tsc --noEmit`.
- Local end-to-end (this PC): build a signed fake `v0.1.1` package with the real key, serve a `latest.json` that points to it from a local HTTP server, start a debug MacDows with `MACDOWS_UPDATE_ENDPOINT`, and check: the dot appears, the card shows the version and notes, **Update and restart** installs it and MacDows runs again at the new version. Also check a wrong signature (tampered file) is rejected, and that an unreachable endpoint shows the one-sentence error on a manual check only.
- To verify first (it decides one detail): whether the NSIS update package relaunches MacDows by itself after installing. If it does not, the plan adds a relaunch step.
- Not verifiable here: a real update from a published GitHub release (none exists yet), and behavior on a PC with a different proxy setup.

## Out of scope
Fully automatic install, pre-release channels, delta updates, downgrade, update UI in the tray menu.

## Risks
- The first published release can only be updated *from* by builds that already contain this feature; v0.1.0 as built before this change cannot update itself.
- The check uses the same endpoint for every user; if a release is published without `latest.json`, the check reports "up to date" or an error depending on GitHub's response.
