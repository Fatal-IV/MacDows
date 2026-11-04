# Control Center Network Pages Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The island's Quick settings screen (the Control Center) gets a Wi-Fi page, a VPN page and a Bluetooth pairing flow: connect to another network (also hidden ones, with a password typed in the island), forget networks, connect Windows VPNs, pair and unpair Bluetooth devices.

**Architecture:** Three Rust modules (`wifi.rs` on Win32 `wlanapi`, `vpn.rs` on RAS plus the adapter list, the existing `bluetooth.rs` extended with a `DeviceWatcher` and custom pairing) expose small commands; pure parts (signal bars, sorting, profile XML, error mapping, pairing-kind mapping, VPN state) are unit-tested. The React side adds pages in the pattern of `BluetoothPage.tsx`; keyboard input in the island goes through one `island_focus` command that makes the notch window take focus while a text field is active and keeps the island from closing.

**Tech Stack:** Rust (`windows` 0.61 features `Win32_NetworkManagement_WiFi`, `Win32_NetworkManagement_Rras`, `Win32_NetworkManagement_IpHelper`, `Devices_Enumeration`, `Devices_Bluetooth`), React 19 + TypeScript, `cargo test`, `node --test`.

**Spec:** `docs/superpowers/specs/2026-11-04-control-center-network-design.md`

## Global Constraints

- The Dock's Windows Quick settings menu is untouched; only the island's Quick settings screen changes.
- New switches under `island.screens.quick`: `wifi_page`, `vpn`, `bluetooth_pairing`, all default true. When off, the old behavior stays (tile only; "Add device" opens Windows settings).
- A Wi-Fi password lives only in memory for the duration of the connect call: never in settings, logs (`eprintln!`), events or error texts.
- Connecting, disconnecting and forgetting only act on what the user clicked. Real tests: only the user's phone hotspot and a nearby pairing device the user names; the current connection is never cut without asking.
- No `eprintln!` of SSIDs with passwords, no network credentials in test fixtures (use `"TestNet"`, `"secret123"`).
- All strings in `src/locales/en.json` and `tr.json` (keys under `net.`); commit messages lowercase conventional, author Fatal-IV, no Claude lines.
- Real-dock verification each task: release build (`bun run tauri build --no-bundle`, stop `macdows` first), run, reveal the island with the pointer at the top centre, capture only the island area.

## Review Focus

- Wi-Fi turned off or no adapter (desktop PC): page shows "Wi-Fi off" / "Not available", no crash, no empty list flicker. (Tasks 1, 2)
- SSID with non-ASCII characters, emoji or XML special characters (`&`, `<`) → profile XML is escaped, connecting works, the name displays right. (Task 1)
- Wrong password on a new network → the row shows "Wrong password", the previous connection is restored. (Task 3)
- The pointer leaves the island while typing a password → the island stays open; focus moves to another window → typing ends and the island behaves normally. (Task 3)
- Bluetooth pairing request while the page is closed → declined safely, no stuck dialog. (Task 5)
- A VPN profile that needs credentials → the row says so and offers "Open Windows settings". (Task 4)

---

## File Structure

- `src-tauri/src/wifi.rs` — create: adapters, scan, networks, connect, disconnect, forget, profile XML.
- `src-tauri/src/vpn.rs` — create: RAS entries and connections, third-party tunnel detection.
- `src-tauri/src/bluetooth.rs` — modify: watcher, pairing, unpair.
- `src-tauri/src/island_settings.rs`, `src/bridge.ts`, `src/notch/islandDefaults.ts` — modify: the three switches.
- `src-tauri/src/lib.rs` — modify: commands, `island_focus`.
- `src/notch/QuickSettings.tsx` — modify: Wi-Fi and VPN cards with arrows, page routing.
- `src/notch/WifiPage.tsx`, `VpnPage.tsx`, `PairPage.tsx` (Bluetooth scan/pair), `netLogic.ts` (+ test) — create.
- `src/notch/DynamicIsland.tsx` — modify: keep the island open while a text field is active.
- locales, `src/settings/IslandSettings.tsx`, `CHANGELOG.md` — modify.

---

### Task 1: Wi-Fi backend: adapters, scan, list, profile XML (no connecting yet)

**Files:**
- Create: `src-tauri/src/wifi.rs`, `src/notch/netLogic.ts`, `src/notch/netLogic.test.ts`
- Modify: `src-tauri/Cargo.toml` (feature `Win32_NetworkManagement_WiFi`), `src-tauri/src/lib.rs`, `src/bridge.ts`
- Test: `src-tauri/src/wifi.rs`, `src/notch/netLogic.test.ts`

**Interfaces:**
- Produces (Rust): `#[derive(Serialize)] pub struct WifiNetwork { ssid: String, signal: u8 /* 0-100 */, secured: bool, enterprise: bool, saved: bool, connected: bool }`; `#[derive(Serialize)] pub struct WifiState { available: bool, enabled: bool, networks: Vec<WifiNetwork> }`; pure `pub fn signal_bars(quality: u8) -> u8` (0-4); pure `pub fn sort_networks(Vec<WifiNetwork>) -> Vec<WifiNetwork>` (connected first, then by signal descending, then name); pure `pub fn profile_xml(ssid: &str, auth: Auth, password: Option<&str>, hidden: bool) -> String` with `pub enum Auth { Open, Wpa2Personal, Wpa3Personal }` and XML escaping; commands `wifi_state() -> WifiState` (async, `spawn_blocking`), `wifi_scan()`.
- Produces (TS): `WifiNetwork`, `WifiState` in `bridge.ts`; `signalBars(signal: number): number`, `securityLabel(n: WifiNetwork, t): string`.

- [ ] **Step 1: Failing tests** — Rust: `signal_bars` thresholds (0→0, 1-25→1, 26-50→2, 51-75→3, 76-100→4); `sort_networks` puts connected first then stronger, ties by lowercase name; `profile_xml` for `Wpa2Personal` contains `WPA2PSK`, `AES` and the key material, for `Open` contains `open` and no `sharedKey`, `hidden` adds `<nonBroadcast>true</nonBroadcast>`, an SSID `A&B <1>` is escaped as `A&amp;B &lt;1&gt;`, `Wpa3Personal` contains `WPA3SAE`. TS: `signalBars` mapping.
- [ ] **Step 2: Run** `cargo test wifi`, `node --test src/notch/netLogic.test.ts` — Expected: FAIL.
- [ ] **Step 3: Implement** with `WlanOpenHandle`, `WlanEnumInterfaces` (no interface → `available: false`), `WlanGetAvailableNetworkList` (flags `HAS_PROFILE`, `CONNECTED`; security via `dot11DefaultAuthAlgorithm`: 802.1X/RSNA-enterprise → `enterprise`), `WlanScan`; the SSID is the raw bytes decoded as UTF-8 (lossy); duplicates by SSID keep the strongest.
- [ ] **Step 4: Run** all tests, `tsc`; add an `#[ignore]` test `prints_the_wifi_networks` and run it by hand — Expected: lists real networks with signal, saved/connected flags; report what it shows.
- [ ] **Step 5: Commit** `feat: Wi-Fi networks from Windows (list, signal, saved and connected state)`.

### Task 2: Wi-Fi page: list, connect to saved or open networks, disconnect, forget

**Files:**
- Create: `src/notch/WifiPage.tsx`
- Modify: `src-tauri/src/wifi.rs` (`wifi_connect_saved`, `wifi_connect_open`, `wifi_disconnect`, `wifi_forget`), `src-tauri/src/lib.rs`, `src/notch/QuickSettings.tsx`, `src/bridge.ts`, locales, `src-tauri/src/island_settings.rs` + `src/bridge.ts` + `islandDefaults.ts` (`wifi_page`)

**Interfaces:**
- Produces: commands `wifi_connect(ssid: String) -> Result<(), String>` (saved profile or open network: `WlanConnect` with the profile name; for an open unsaved network the open profile XML is saved first), `wifi_disconnect()`, `wifi_forget(ssid: String)` (`WlanDeleteProfile`, refuses while connected to it? no: disconnects first); error texts are fixed keys (`"not_found"`, `"failed"`, `"timeout"`) mapped to locale strings by `netLogic.errorText`.
- UI: `WifiPage({ onBack })` polls `wifi_state` every 5 s while mounted (after calling `wifi_scan`); rows per spec §2.1; the Wi-Fi card in `QuickSettings.tsx` gets the `›` arrow when `island.screens.quick.wifi_page`.

- [ ] **Step 1: Failing tests** — `error_key(code: u32) -> &'static str` (reason codes to `not_found`/`failed`/`timeout`), `netLogic.errorText` returns a locale key for every error key and falls back for unknown ones.
- [ ] **Step 2: Run** the tests — Expected: FAIL.
- [ ] **Step 3: Implement** commands (wait for the connection result for up to 10 s through `WlanRegisterNotification`, or poll `WlanQueryInterface` for the connected state), the page and the card arrow, forget confirmation row ("Forget this network?" Yes/Cancel, in-page, no browser dialog).
- [ ] **Step 4: Verify** tests, `tsc`, release build; on the real PC with the user's agreement: open the page, check the list and signals match Windows; **do not connect elsewhere**; reconnect to the current network only if the user says so. Capture the island.
- [ ] **Step 5: Commit** `feat: Wi-Fi page in the control center (connect to saved and open networks, disconnect, forget)`.

### Task 3: Password entry, new secured networks, hidden networks, keyboard focus

**Files:**
- Modify: `src-tauri/src/wifi.rs` (`wifi_connect_new(ssid, password, auth, hidden)`), `src-tauri/src/lib.rs` (`island_focus`), `src/notch/WifiPage.tsx`, `src/notch/DynamicIsland.tsx`, `src-tauri/src/visibility.rs` (hold while typing), locales

**Interfaces:**
- Produces: `wifi_connect_new(ssid: String, password: Option<String>, auth: String, hidden: bool) -> Result<(), String>` (`auth` is `"open" | "wpa2" | "wpa3"`; saves the profile with `profile_xml`, connects, on failure deletes the new profile and reconnects the previous profile if there was one; error key `"wrong_password"` when the failure reason is a PSK mismatch); `island_focus(on: bool)` (notch window `set_focus` / back to no-focus; also `visibility::hold_notch` while on).
- UI: password row (secured unsaved network) with `type="password"` input, Connect button, Enter/Esc; "Join a hidden network" form: SSID, security select, password. While any text field has focus the island ignores `onHoverEnd` (a ref `editing`); window blur ends editing.

- [ ] **Step 1: Failing tests** — `profile_xml` already covered; add `error_key` mapping for a PSK failure code → `wrong_password`; `netLogic.canSubmit(ssid, auth, password)` (SSID required; WPA2 needs 8-63 characters, WPA3 at least 1, open needs none) with its edge cases (7 chars rejected, 64 rejected, spaces allowed).
- [ ] **Step 2: Run** — Expected: FAIL.
- [ ] **Step 3: Implement** the command, focus handling and the forms.
- [ ] **Step 4: Verify:** unit tests; real test on the user's phone hotspot (the user turns it on and gives the password then): connect from the island, then, if the user agrees, try a wrong password once and check the error row and that the previous network came back; test the focus rules (pointer leaves while typing → stays; click elsewhere → closes normally).
- [ ] **Step 5: Commit** `feat: join Wi-Fi networks with a password typed in the island, including hidden networks`.

### Task 4: VPN card and page

**Files:**
- Create: `src-tauri/src/vpn.rs`, `src/notch/VpnPage.tsx`
- Modify: `src-tauri/Cargo.toml` (`Win32_NetworkManagement_Rras`, `Win32_NetworkManagement_IpHelper`), `src-tauri/src/lib.rs`, `src/notch/QuickSettings.tsx`, `src/bridge.ts`, settings model (`vpn`), locales
- Test: `src-tauri/src/vpn.rs`

**Interfaces:**
- Produces: `#[derive(Serialize)] pub struct VpnEntry { name: String, state: String /* "connected" | "disconnected" | "connecting" */ }`, `#[derive(Serialize)] pub struct VpnState { entries: Vec<VpnEntry>, app_vpn_active: bool }`; pure `pub fn state_of(ras_state: u32) -> &'static str`; pure `pub fn is_tunnel_adapter(if_type: u32, description: &str) -> bool` (tunnel type 131 or names like WireGuard/TAP/Wintun/NordLynx); commands `vpn_state`, `vpn_connect(name)`, `vpn_disconnect(name)`.

- [ ] **Step 1: Failing tests** — `state_of` for RAS connected/connecting/disconnected states, `is_tunnel_adapter` for tunnel type and known names and a plain Ethernet adapter.
- [ ] **Step 2: Run** `cargo test vpn` — Expected: FAIL.
- [ ] **Step 3: Implement** with `RasEnumEntriesW`, `RasEnumConnectionsW`, `RasDialW` (stored credentials only; a failure that needs credentials returns the key `"credentials"`), `RasHangUpW`; the card shows Off / On / Connecting / "On (app)"; the page lists entries or the empty state with the Windows settings link.
- [ ] **Step 4: Verify:** tests; the real PC has no VPN, so check the empty state and the card; report that connect/disconnect could not be exercised for lack of a profile.
- [ ] **Step 5: Commit** `feat: VPN card and page in the control center (Windows VPN connections, app VPN status)`.

### Task 5: Bluetooth scan and pairing

**Files:**
- Create: `src/notch/PairPage.tsx`
- Modify: `src-tauri/src/bluetooth.rs`, `src-tauri/src/lib.rs`, `src/notch/BluetoothPage.tsx`, `src/bridge.ts`, settings model (`bluetooth_pairing`), locales
- Test: `src-tauri/src/bluetooth.rs`

**Interfaces:**
- Produces: pure `pub fn pairing_kind(kinds: u32) -> &'static str` mapping `DevicePairingKinds` flags to `"confirm" | "confirm_pin" | "provide_pin" | "none"` (the strongest requirement wins); pure `pub fn visible_in_scan(name: &str, paired: bool) -> bool` (named, unpaired devices only); commands `bt_scan_start`, `bt_scan_stop`, `bt_pair(id: String) -> Result<(), String>`, `bt_pairing_answer(accept: bool, pin: Option<String>)`; events `bt-found` (`{ id, name, kind }`), `bt-lost` (`{ id }`), `bt-pairing-request` (`{ id, kind, pin: Option<String> }`), `bt-paired` (`{ id }`).
- Pairing runs on its own thread with a 30 s timeout; a request with no listening page is declined.

- [ ] **Step 1: Failing tests** — `pairing_kind` table (ConfirmOnly=1, DisplayPin=2, ProvidePin=4, ConfirmPinMatch=8, combinations, 0), `visible_in_scan` (empty name hidden, paired hidden, named unpaired shown).
- [ ] **Step 2: Run** `cargo test bluetooth` — Expected: FAIL.
- [ ] **Step 3: Implement** the watcher (started when the page opens, stopped when it closes or the island hides), the pairing flow and `PairPage` ("Nearby devices" list, confirm / PIN match / PIN entry rows, progress and result per row); the "Add device" button opens it when `bluetooth_pairing` is on.
- [ ] **Step 4: Verify:** tests; real scan with the user's phone or earbuds in pairing mode (the user names the device); pair only that one.
- [ ] **Step 5: Commit** `feat: pair new Bluetooth devices from the control center`.

### Task 6: Unpair, settings switches, texts, changelog

**Files:**
- Modify: `src-tauri/src/bluetooth.rs` (`bt_unpair`), `src/notch/BluetoothPage.tsx`, `src/settings/IslandSettings.tsx` (Quick settings section: three switches), locales, `CHANGELOG.md`

- [ ] **Step 1:** `bt_unpair(id)` with `UnpairAsync` and an in-page confirmation ("Remove this device?"); the paired list refreshes.
- [ ] **Step 2:** Quick settings section rows `wifi_page`, `vpn`, `bluetooth_pairing`; locale completeness check (`node -e` key diff prints no differences).
- [ ] **Step 3:** CHANGELOG entry; final verification: `cargo test`, `node --test src/*/*.test.ts`, `tsc`, release build, one pass over every page on the real PC.
- [ ] **Step 4: Commit** `feat: unpair Bluetooth devices, switches and texts for the network pages`.

---

## Self-Review

- **Spec coverage:** §2.1 Wi-Fi page → Tasks 1-3 (hidden network form and forget included); §2.2 focus → Task 3; §2.3 Bluetooth → Tasks 5-6; §2.4 VPN → Task 4; §3 switches → Tasks 2, 4, 5, 6; §4 modules and commands → Tasks 1-5; §5 phases map 1:1 to Tasks 1-6; §6 out of scope respected (no VPN profile editing, no third-party VPN control).
- **Type consistency:** `WifiNetwork`/`WifiState` (Task 1) are used by the page in Task 2 and extended by nothing; `profile_xml` and `Auth` (Task 1) are used by `wifi_connect_new` (Task 3); `island_focus` (Task 3) is reused by the Bluetooth PIN entry (Task 5); the three switches share the `quick` settings struct from the Dynamic Island settings model.
- **Open for the user:** whether Wi-Fi connect failures should restore the previous network automatically (the plan says yes).
