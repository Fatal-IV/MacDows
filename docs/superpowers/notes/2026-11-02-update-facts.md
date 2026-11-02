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
