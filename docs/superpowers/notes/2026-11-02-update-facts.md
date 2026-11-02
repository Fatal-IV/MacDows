# In-app update facts (2026-11-02)

## From the plugin source (`tauri-plugin-updater` 2.14.1, `config.rs`, `updater.rs`)
- A non-`https` endpoint is **allowed in debug builds** (it only prints a warning) and **refused in release builds** (`InsecureTransportProtocol`) unless `dangerousInsecureTransportProtocol` is set. So the debug-only `MACDOWS_UPDATE_ENDPOINT` override in the spec needs no extra configuration, and a release build can never be pointed at plain http.
- The current version used for the comparison is the app's own version (`package_info().version`, from `tauri.conf.json`).
- `UpdaterBuilder::endpoints(Vec<Url>)` replaces the configured endpoints.

## Fake release for the local test
- A signed fake `0.1.1` package was built with the real key: `src-tauri/target/release/bundle/nsis/MacDows_0.1.1_x64-setup.exe` (2,479,641 bytes) and its `.sig`; the source tree stayed untouched (`--config '{"version":"0.1.1"}'`).
- `node scripts/fake-update-server.mjs <dir> 0.1.1 8787` serves `/latest.json` (200, JSON with `version`, `notes`, `pub_date`, `platforms."windows-x86_64".{signature,url}`) and `/package.exe` (200, full size); unknown paths answer 404. `--tamper` flips the first byte of the served package.

## Still to be answered by the end-to-end run (Task 5)
- Whether the NSIS update package relaunches MacDows after installing (observe `macdows.exe` running from `%LOCALAPPDATA%\MacDows` after the update).

## End-to-end run on this PC (2026-11-02)
Debug MacDows 0.1.0 pointed at a local fake release server (`MACDOWS_UPDATE_ENDPOINT`), fake signed 0.1.1 package, per the user's permission. Settings were backed up and restored afterwards; the installed test copy was uninstalled.
- **Notch dot:** after the background check found 0.1.1, a blue dot appeared on the right of the notch pill (screenshot checked).
- **Settings card:** "Güncellemeler" with the installed version 0.1.0, status, release notes ("Yenilikler"), buttons "Şimdi denetle" and "Güncelle ve yeniden başlat", and the automatic-check switch. "Check now" turned the status into "0.1.1 sürümü hazır." (screenshot checked).
- **Install:** pressing "Güncelle ve yeniden başlat" installed 0.1.1 in about 4 seconds (registry `DisplayVersion` 0.1.1, exe product version 0.1.1). **The NSIS update package relaunched MacDows by itself** from `%LOCALAPPDATA%\MacDows\macdows.exe`, so no relaunch step is needed.
- **Tampered package** (first byte changed): not installed; the card showed "Güncelleme doğrulanamadı ve kurulmadı." in the error style; nothing was installed; the app kept running; the log says `install failed: The signature verification failed`.
- **Server down, manual check:** "Güncellemeler denetlenemedi. Bağlantınızı kontrol edip yeniden deneyin." (one sentence, error style); the log has the technical detail.
- **Server down, background check:** ran 30 s after launch, wrote a log line, showed no error ("Henüz denetlenmedi."), and did not record `last_update_check`, so it is retried at the next poll.
- A blank white Settings window was seen once right after the new import `@tauri-apps/api/app` was added: Vite re-optimized that dependency and the already-open page was stale. A reload fixed it, console had no errors; this is a dev-server effect only (release builds bundle everything).
- The Settings window was made 660 px high so the last row is not at the edge.

## Not verified
A real update from a published GitHub release (none exists), proxies, and an update while another user session is active. `last_update_check` skipping within 12 hours was seen working (status stayed "Henüz denetlenmedi." after a restart less than 12 hours after a check).
