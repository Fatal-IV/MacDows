//! Wi-Fi networks for the island's Control Center: adapters, scan, the network list and profile XML.
//!
//! Talks to the Windows WLAN service (`wlanapi`). A Wi-Fi password only ever lives in memory while a profile
//! XML is built: it is never logged and never stored.

use serde::Serialize;
use std::ffi::c_void;
use windows::Win32::Foundation::HANDLE;
use windows::core::PCWSTR;
use windows::Win32::NetworkManagement::WiFi::{
    dot11_BSS_type_any, wlan_connection_mode_profile, WlanCloseHandle, WlanConnect, WlanDeleteProfile, WlanDisconnect, WlanEnumInterfaces, WlanFreeMemory,
    WlanGetAvailableNetworkList, WlanOpenHandle, WlanScan, WlanSetProfile, WLAN_AVAILABLE_NETWORK_LIST, WLAN_CONNECTION_PARAMETERS, WLAN_INTERFACE_INFO_LIST,
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct WifiNetwork {
    pub ssid: String,
    /// 0-100.
    pub signal: u8,
    pub secured: bool,
    pub enterprise: bool,
    pub saved: bool,
    pub connected: bool,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct WifiState {
    /// False when the PC has no Wi-Fi adapter (or the WLAN service is missing).
    pub available: bool,
    pub enabled: bool,
    pub networks: Vec<WifiNetwork>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Auth {
    Open,
    Wpa2Personal,
    Wpa3Personal,
}

/// 0-100 signal quality to 0-4 bars.
pub fn signal_bars(quality: u8) -> u8 {
    match quality {
        0 => 0,
        1..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        _ => 4,
    }
}

/// Connected first, then the stronger signal, ties by lowercase name.
pub fn sort_networks(mut list: Vec<WifiNetwork>) -> Vec<WifiNetwork> {
    list.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.signal.cmp(&a.signal))
            .then(a.ssid.to_lowercase().cmp(&b.ssid.to_lowercase()))
    });
    list
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

/// The WLAN profile XML Windows wants for `WlanSetProfile`.
pub fn profile_xml(ssid: &str, auth: Auth, password: Option<&str>, hidden: bool) -> String {
    let name = xml_escape(ssid);
    let (authentication, encryption) = match auth {
        Auth::Open => ("open", "none"),
        Auth::Wpa2Personal => ("WPA2PSK", "AES"),
        Auth::Wpa3Personal => ("WPA3SAE", "AES"),
    };
    let non_broadcast = if hidden { "<nonBroadcast>true</nonBroadcast>" } else { "" };
    let key = match (auth, password) {
        (Auth::Open, _) | (_, None) => String::new(),
        (_, Some(p)) => format!(
            "<sharedKey><keyType>passPhrase</keyType><protected>false</protected><keyMaterial>{}</keyMaterial></sharedKey>",
            xml_escape(p)
        ),
    };
    format!(
        "<?xml version=\"1.0\"?><WLANProfile xmlns=\"http://www.microsoft.com/networking/WLAN/profile/v1\"><name>{name}</name>\
<SSIDConfig><SSID><name>{name}</name></SSID>{non_broadcast}</SSIDConfig><connectionType>ESS</connectionType><connectionMode>auto</connectionMode>\
<MSM><security><authEncryption><authentication>{authentication}</authentication><encryption>{encryption}</encryption><useOneX>false</useOneX></authEncryption>{key}</security></MSM></WLANProfile>"
    )
}

struct Client(HANDLE);
impl Drop for Client {
    fn drop(&mut self) {
        unsafe { WlanCloseHandle(self.0, None) };
    }
}

fn open() -> Option<Client> {
    let mut negotiated = 0u32;
    let mut handle = HANDLE::default();
    let err = unsafe { WlanOpenHandle(2, None, &mut negotiated, &mut handle) };
    if err != 0 {
        return None;
    }
    Some(Client(handle))
}

/// The interface GUIDs of all Wi-Fi adapters.
fn interfaces(client: &Client) -> Vec<windows::core::GUID> {
    let mut out = Vec::new();
    let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
    let err = unsafe { WlanEnumInterfaces(client.0, None, &mut list) };
    if err != 0 || list.is_null() {
        return out;
    }
    unsafe {
        let count = (*list).dwNumberOfItems as usize;
        let first = (*list).InterfaceInfo.as_ptr();
        for i in 0..count {
            out.push((*first.add(i)).InterfaceGuid);
        }
        WlanFreeMemory(list as *const c_void);
    }
    out
}

fn is_enterprise(algo: i32) -> bool {
    // 802.1X family: WPA (3), RSNA (6), WPA3 enterprise (8).
    matches!(algo, 3 | 6 | 8)
}

fn merge(into: &mut Vec<WifiNetwork>, net: WifiNetwork) {
    match into.iter_mut().find(|e| e.ssid == net.ssid) {
        Some(e) => {
            e.signal = e.signal.max(net.signal);
            e.saved |= net.saved;
            e.connected |= net.connected;
        }
        None => into.push(net),
    }
}

fn read_networks(client: &Client, guid: &windows::core::GUID) -> Option<Vec<WifiNetwork>> {
    let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
    // Flags 0: the list still carries HAS_PROFILE and CONNECTED per network.
    let err = unsafe { WlanGetAvailableNetworkList(client.0, guid, 0, None, &mut list) };
    if err != 0 || list.is_null() {
        return None;
    }
    let mut out: Vec<WifiNetwork> = Vec::new();
    unsafe {
        let count = (*list).dwNumberOfItems as usize;
        let first = (*list).Network.as_ptr();
        for i in 0..count {
            let n = &*first.add(i);
            let len = (n.dot11Ssid.uSSIDLength as usize).min(32);
            let ssid = String::from_utf8_lossy(&n.dot11Ssid.ucSSID[..len]).to_string();
            if ssid.is_empty() {
                continue;
            }
            let secured = n.bSecurityEnabled.as_bool();
            merge(
                &mut out,
                WifiNetwork {
                    ssid,
                    signal: n.wlanSignalQuality.min(100) as u8,
                    secured,
                    enterprise: secured && is_enterprise(n.dot11DefaultAuthAlgorithm.0),
                    saved: n.dwFlags & 0x2 != 0,
                    connected: n.dwFlags & 0x1 != 0,
                },
            );
        }
        WlanFreeMemory(list as *const c_void);
    }
    Some(out)
}

pub fn current_state() -> WifiState {
    let Some(client) = open() else { return WifiState::default() };
    let guids = interfaces(&client);
    if guids.is_empty() {
        return WifiState::default();
    }
    let mut merged: Vec<WifiNetwork> = Vec::new();
    let mut enabled = false;
    for g in &guids {
        if let Some(list) = read_networks(&client, g) {
            enabled = true;
            for n in list {
                merge(&mut merged, n);
            }
        }
    }
    WifiState { available: true, enabled, networks: sort_networks(merged) }
}

fn request_scan() {
    let Some(client) = open() else { return };
    for g in interfaces(&client) {
        unsafe { WlanScan(client.0, &g, None, None, None) };
    }
}

/// Fixed error keys the page maps to texts: "not_found", "failed", "timeout" (plus "needs_password" and "enterprise" for networks that cannot be joined this way).
pub fn error_key(code: u32) -> &'static str {
    match code {
        // ERROR_NOT_FOUND, ERROR_INVALID_PARAMETER (no such profile)
        1168 | 87 => "not_found",
        // ERROR_TIMEOUT, WAIT_TIMEOUT
        1460 | 258 => "timeout",
        _ => "failed",
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Where a visible network lives: its adapter, whether a profile exists (and its name) and how it is secured.
struct Found {
    guid: windows::core::GUID,
    profile: Option<String>,
    secured: bool,
    enterprise: bool,
}

fn find_network(client: &Client, ssid: &str) -> Option<Found> {
    for guid in interfaces(client) {
        let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
        if unsafe { WlanGetAvailableNetworkList(client.0, &guid, 0, None, &mut list) } != 0 || list.is_null() {
            continue;
        }
        let mut found = None;
        unsafe {
            let count = (*list).dwNumberOfItems as usize;
            let first = (*list).Network.as_ptr();
            for i in 0..count {
                let n = &*first.add(i);
                let len = (n.dot11Ssid.uSSIDLength as usize).min(32);
                if String::from_utf8_lossy(&n.dot11Ssid.ucSSID[..len]) != ssid {
                    continue;
                }
                let name_len = n.strProfileName.iter().position(|&c| c == 0).unwrap_or(n.strProfileName.len());
                let profile = (n.dwFlags & 0x2 != 0).then(|| String::from_utf16_lossy(&n.strProfileName[..name_len]));
                let secured = n.bSecurityEnabled.as_bool();
                found = Some(Found { guid, profile, secured, enterprise: secured && is_enterprise(n.dot11DefaultAuthAlgorithm.0) });
                break;
            }
            WlanFreeMemory(list as *const c_void);
        }
        if found.is_some() {
            return found;
        }
    }
    None
}

fn is_connected_to(ssid: &str) -> bool {
    current_state().networks.iter().any(|n| n.connected && n.ssid == ssid)
}

/// Asks Windows to connect with the named profile and waits up to 10 s for the connected state.
fn connect_profile(client: &Client, guid: &windows::core::GUID, profile: &str, ssid: &str) -> Result<(), String> {
    let name = wide(profile);
    let params = WLAN_CONNECTION_PARAMETERS {
        wlanConnectionMode: wlan_connection_mode_profile,
        strProfile: PCWSTR(name.as_ptr()),
        pDot11Ssid: std::ptr::null_mut(),
        pDesiredBssidList: std::ptr::null_mut(),
        dot11BssType: dot11_BSS_type_any,
        dwFlags: 0,
    };
    let err = unsafe { WlanConnect(client.0, guid, &params, None) };
    if err != 0 {
        return Err(error_key(err).to_string());
    }
    for _ in 0..20 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if is_connected_to(ssid) {
            return Ok(());
        }
    }
    Err("timeout".to_string())
}

fn connect_blocking(ssid: &str) -> Result<(), String> {
    let client = open().ok_or("failed")?;
    let found = find_network(&client, ssid).ok_or("not_found")?;
    if found.enterprise {
        return Err("enterprise".into());
    }
    let profile = match found.profile {
        Some(p) => p,
        None if !found.secured => {
            // An open network without a profile: save one, then connect with it.
            let xml = wide(&profile_xml(ssid, Auth::Open, None, false));
            let mut reason = 0u32;
            let err = unsafe { WlanSetProfile(client.0, &found.guid, 0, PCWSTR(xml.as_ptr()), PCWSTR::null(), true, None, &mut reason) };
            if err != 0 {
                return Err(error_key(err).to_string());
            }
            ssid.to_string()
        }
        None => return Err("needs_password".into()),
    };
    connect_profile(&client, &found.guid, &profile, ssid)
}

fn disconnect_blocking() {
    let Some(client) = open() else { return };
    for g in interfaces(&client) {
        unsafe { WlanDisconnect(client.0, &g, None) };
    }
}

fn forget_blocking(ssid: &str) -> Result<(), String> {
    let client = open().ok_or("failed")?;
    let found = find_network(&client, ssid).ok_or("not_found")?;
    let profile = found.profile.ok_or("not_found")?;
    if is_connected_to(ssid) {
        unsafe { WlanDisconnect(client.0, &found.guid, None) };
    }
    let name = wide(&profile);
    let err = unsafe { WlanDeleteProfile(client.0, &found.guid, PCWSTR(name.as_ptr()), None) };
    if err != 0 {
        return Err(error_key(err).to_string());
    }
    Ok(())
}

/// Connects to a saved or open network the user clicked.
#[tauri::command]
pub async fn wifi_connect(ssid: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || connect_blocking(&ssid)).await.map_err(|_| "failed".to_string())?
}

#[tauri::command]
pub async fn wifi_disconnect() {
    let _ = tauri::async_runtime::spawn_blocking(disconnect_blocking).await;
}

/// Removes the saved profile of a network (disconnecting first when it is the current one).
#[tauri::command]
pub async fn wifi_forget(ssid: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || forget_blocking(&ssid)).await.map_err(|_| "failed".to_string())?
}

#[tauri::command]
pub async fn wifi_state() -> WifiState {
    tauri::async_runtime::spawn_blocking(current_state).await.unwrap_or_default()
}

/// Asks the adapters for a fresh scan; the list updates a few seconds later.
#[tauri::command]
pub async fn wifi_scan() {
    let _ = tauri::async_runtime::spawn_blocking(request_scan).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net(ssid: &str, signal: u8, connected: bool) -> WifiNetwork {
        WifiNetwork { ssid: ssid.into(), signal, secured: true, enterprise: false, saved: false, connected }
    }

    #[test]
    fn bars_follow_the_thresholds() {
        let cases = [(0, 0), (1, 1), (25, 1), (26, 2), (50, 2), (51, 3), (75, 3), (76, 4), (100, 4)];
        for (q, b) in cases {
            assert_eq!(signal_bars(q), b, "quality {q}");
        }
    }

    #[test]
    fn sorting_puts_connected_first_then_strongest_then_name() {
        let sorted = sort_networks(vec![net("b", 50, false), net("A", 50, false), net("weak", 10, true), net("strong", 90, false)]);
        let names: Vec<&str> = sorted.iter().map(|n| n.ssid.as_str()).collect();
        assert_eq!(names, ["weak", "strong", "A", "b"]);
    }

    #[test]
    fn wpa2_profile_has_the_key() {
        let xml = profile_xml("TestNet", Auth::Wpa2Personal, Some("secret123"), false);
        assert!(xml.contains("WPA2PSK") && xml.contains("AES") && xml.contains("<keyMaterial>secret123</keyMaterial>"));
        assert!(!xml.contains("nonBroadcast"));
    }

    #[test]
    fn open_profile_has_no_key() {
        let xml = profile_xml("TestNet", Auth::Open, Some("ignored"), false);
        assert!(xml.contains("open") && !xml.contains("sharedKey") && !xml.contains("ignored"));
    }

    #[test]
    fn hidden_and_wpa3_and_escaping() {
        assert!(profile_xml("TestNet", Auth::Open, None, true).contains("<nonBroadcast>true</nonBroadcast>"));
        assert!(profile_xml("TestNet", Auth::Wpa3Personal, Some("secret123"), false).contains("WPA3SAE"));
        assert!(profile_xml("A&B <1>", Auth::Open, None, false).contains("A&amp;B &lt;1&gt;"));
    }

    #[test]
    fn reason_codes_map_to_error_keys() {
        assert_eq!(error_key(1168), "not_found");
        assert_eq!(error_key(87), "not_found");
        assert_eq!(error_key(1460), "timeout");
        assert_eq!(error_key(5), "failed");
        assert_eq!(error_key(99999), "failed");
    }

    #[test]
    #[ignore]
    fn prints_the_wifi_networks() {
        let s = current_state();
        println!("available={} enabled={}", s.available, s.enabled);
        for n in s.networks {
            println!(
                "{:<32} signal={:>3} bars={} secured={} ent={} saved={} connected={}",
                n.ssid, n.signal, signal_bars(n.signal), n.secured, n.enterprise, n.saved, n.connected
            );
        }
    }
}
