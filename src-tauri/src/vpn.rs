//! VPN for the island's Control Center: the VPN connections set up in Windows (RAS: connect, disconnect) and whether a
//! third-party VPN app (WireGuard, OpenVPN, ...) has a tunnel up. App VPNs are only reported, never controlled.
//! Only stored credentials are used; a profile that needs a password typed reports `"credentials"`.

use serde::Serialize;
use windows::core::{BOOL, PCWSTR};
use windows::Win32::NetworkManagement::IpHelper::{GetAdaptersAddresses, GET_ADAPTERS_ADDRESSES_FLAGS, IP_ADAPTER_ADDRESSES_LH};
use windows::Win32::NetworkManagement::Rras::{
    RasDialW, RasEnumConnectionsW, RasEnumEntriesW, RasGetConnectStatusW, RasGetEntryDialParamsW, RasHangUpW, HRASCONN, RASCONNSTATUSW, RASCONNW, RASDIALPARAMSW,
    RASENTRYNAMEW,
};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct VpnEntry {
    pub name: String,
    /// "connected", "disconnected" or "connecting".
    pub state: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct VpnState {
    pub entries: Vec<VpnEntry>,
    /// A third-party VPN tunnel adapter is up.
    pub app_vpn_active: bool,
}

/// RAS connection state (`RASCS_*`) to the three states the island shows.
pub fn state_of(ras_state: u32) -> &'static str {
    match ras_state {
        0x2000 => "connected",
        0x2001 => "disconnected",
        _ => "connecting",
    }
}

/// Whether a network adapter is a VPN tunnel of an app: the tunnel interface type, or a well-known driver name.
/// Windows's own IPv6 transition tunnels (Teredo, ISATAP, 6to4) have the same interface type and are not VPNs.
pub fn is_tunnel_adapter(if_type: u32, description: &str) -> bool {
    let d = description.to_lowercase();
    if ["teredo", "isatap", "6to4", "tunneling pseudo", "iphttps"].iter().any(|w| d.contains(w)) {
        return false;
    }
    if if_type == 131 {
        return true;
    }
    ["wireguard", "wintun", "nordlynx", "tap-windows", "tap-nordvpn", "openvpn", "tailscale", "proton"].iter().any(|w| d.contains(w))
}

fn to_string(wide: &[u16]) -> String {
    let len = wide.iter().position(|&c| c == 0).unwrap_or(wide.len());
    String::from_utf16_lossy(&wide[..len])
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn entry_names() -> Vec<String> {
    let mut size = 0u32;
    let mut count = 0u32;
    // First call with no buffer reports the size needed (ERROR_BUFFER_TOO_SMALL, 603); no entries gives success and zero.
    let err = unsafe { RasEnumEntriesW(PCWSTR::null(), PCWSTR::null(), None, &mut size, &mut count) };
    if (err != 603 && err != 0) || size == 0 {
        return Vec::new();
    }
    let n = (size as usize).div_ceil(std::mem::size_of::<RASENTRYNAMEW>()).max(1);
    let mut buf = vec![RASENTRYNAMEW::default(); n];
    buf[0].dwSize = std::mem::size_of::<RASENTRYNAMEW>() as u32;
    let err = unsafe { RasEnumEntriesW(PCWSTR::null(), PCWSTR::null(), Some(buf.as_mut_ptr()), &mut size, &mut count) };
    if err != 0 {
        return Vec::new();
    }
    buf.iter().take(count as usize).map(|e| to_string(&e.szEntryName)).collect()
}

/// Active connections as (entry name, handle).
fn connections() -> Vec<(String, HRASCONN)> {
    let mut size = 0u32;
    let mut count = 0u32;
    let err = unsafe { RasEnumConnectionsW(None, &mut size, &mut count) };
    if (err != 603 && err != 0) || size == 0 {
        return Vec::new();
    }
    let n = (size as usize).div_ceil(std::mem::size_of::<RASCONNW>()).max(1);
    let mut buf = vec![RASCONNW::default(); n];
    buf[0].dwSize = std::mem::size_of::<RASCONNW>() as u32;
    let err = unsafe { RasEnumConnectionsW(Some(buf.as_mut_ptr()), &mut size, &mut count) };
    if err != 0 {
        return Vec::new();
    }
    buf.iter().take(count as usize).map(|c| (to_string(&c.szEntryName), c.hrasconn)).collect()
}

fn connection_state(handle: HRASCONN) -> &'static str {
    let mut status = RASCONNSTATUSW { dwSize: std::mem::size_of::<RASCONNSTATUSW>() as u32, ..Default::default() };
    if unsafe { RasGetConnectStatusW(handle, &mut status) } != 0 {
        return "disconnected";
    }
    state_of(status.rasconnstate.0 as u32)
}

fn app_vpn_active() -> bool {
    let mut size = 15_000u32;
    // u64 elements: the adapter structs need 8-byte alignment, which a Vec<u8> does not promise.
    let mut buf: Vec<u64> = vec![0; (size as usize).div_ceil(8)];
    let flags = GET_ADAPTERS_ADDRESSES_FLAGS(0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x0010);
    let mut err = unsafe { GetAdaptersAddresses(0, flags, None, Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH), &mut size) };
    if err == 111 {
        buf = vec![0; (size as usize).div_ceil(8)];
        err = unsafe { GetAdaptersAddresses(0, flags, None, Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH), &mut size) };
    }
    if err != 0 {
        return false;
    }
    let mut p = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    while !p.is_null() {
        let a = unsafe { &*p };
        // 1 = IfOperStatusUp.
        if a.OperStatus.0 == 1 && is_tunnel_adapter(a.IfType, &unsafe { a.Description.to_string() }.unwrap_or_default()) {
            return true;
        }
        p = a.Next;
    }
    false
}

pub fn current_state() -> VpnState {
    let active = connections();
    let entries = entry_names()
        .into_iter()
        .map(|name| {
            let state = active.iter().find(|(n, _)| *n == name).map(|(_, h)| connection_state(*h)).unwrap_or("disconnected");
            VpnEntry { name, state: state.to_string() }
        })
        .collect();
    VpnState { entries, app_vpn_active: app_vpn_active() }
}

/// Why a dial failed. 691 is access denied (wrong stored credentials); a profile with no stored password that
/// fails to dial needs one typed, which this panel does not offer. Anything else is a plain failure.
pub fn dial_error_key(err: u32, has_password: bool) -> &'static str {
    if err == 691 || !has_password { "credentials" } else { "failed" }
}

fn connect_blocking(name: &str) -> Result<(), String> {
    let mut params = RASDIALPARAMSW { dwSize: std::mem::size_of::<RASDIALPARAMSW>() as u32, ..Default::default() };
    let entry = wide(name);
    let n = entry.len().min(params.szEntryName.len());
    params.szEntryName[..n].copy_from_slice(&entry[..n]);
    let mut has_password = BOOL(0);
    let err = unsafe { RasGetEntryDialParamsW(PCWSTR::null(), &mut params, &mut has_password) };
    if err != 0 {
        return Err("not_found".into());
    }
    // No pre-check for a stored password: certificate (IKEv2) and EAP profiles have none and still dial.
    let has_password = has_password.as_bool();
    let mut handle = HRASCONN::default();
    // No notifier: the call returns once the connection is up or has failed.
    let err = unsafe { RasDialW(None, PCWSTR::null(), &params, 0, None, &mut handle) };
    if err != 0 {
        if !handle.0.is_null() {
            unsafe { RasHangUpW(handle) };
        }
        return Err(dial_error_key(err, has_password).into());
    }
    Ok(())
}

fn disconnect_blocking(name: &str) {
    for (n, handle) in connections() {
        if n != name {
            continue;
        }
        unsafe { RasHangUpW(handle) };
        // The hang-up finishes in the background; wait until the handle is gone (up to 3 s).
        for _ in 0..30 {
            let mut status = RASCONNSTATUSW { dwSize: std::mem::size_of::<RASCONNSTATUSW>() as u32, ..Default::default() };
            if unsafe { RasGetConnectStatusW(handle, &mut status) } != 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

#[tauri::command]
pub async fn vpn_state() -> VpnState {
    tauri::async_runtime::spawn_blocking(current_state).await.unwrap_or_default()
}

#[tauri::command]
pub async fn vpn_connect(name: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || connect_blocking(&name)).await.map_err(|_| "failed".to_string())?
}

#[tauri::command]
pub async fn vpn_disconnect(name: String) {
    let _ = tauri::async_runtime::spawn_blocking(move || disconnect_blocking(&name)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ras_states_map_to_three() {
        assert_eq!(state_of(0x2000), "connected");
        assert_eq!(state_of(0x2001), "disconnected");
        assert_eq!(state_of(0), "connecting"); // OpenPort
        assert_eq!(state_of(14), "connecting"); // Authenticate
        assert_eq!(state_of(0x1000), "connecting"); // Interactive
    }

    #[test]
    fn tunnel_adapters_are_found_by_type_or_name() {
        assert!(is_tunnel_adapter(131, "Some tunnel"));
        assert!(is_tunnel_adapter(53, "WireGuard Tunnel"));
        assert!(is_tunnel_adapter(6, "Wintun Userspace Tunnel"));
        assert!(is_tunnel_adapter(6, "TAP-Windows Adapter V9"));
        assert!(is_tunnel_adapter(6, "NordLynx Tunnel"));
        assert!(!is_tunnel_adapter(6, "Realtek PCIe GbE Family Controller"));
        assert!(!is_tunnel_adapter(71, "Intel(R) Wi-Fi 6 AX201 160MHz"));
    }

    #[test]
    fn the_ip_https_adapter_is_not_a_vpn() {
        assert!(!is_tunnel_adapter(131, "iphttpsinterface"));
    }

    #[test]
    fn a_failed_dial_asks_for_credentials_only_when_that_can_be_the_cause() {
        assert_eq!(dial_error_key(691, true), "credentials");
        assert_eq!(dial_error_key(651, false), "credentials");
        assert_eq!(dial_error_key(651, true), "failed");
    }

    #[test]
    fn windows_own_ipv6_tunnels_are_not_vpns() {
        assert!(!is_tunnel_adapter(131, "Teredo Tunneling Pseudo-Interface"));
        assert!(!is_tunnel_adapter(131, "Microsoft ISATAP Adapter"));
        assert!(!is_tunnel_adapter(131, "Microsoft 6to4 Adapter"));
    }

    #[test]
    #[ignore]
    fn prints_the_vpn_state() {
        println!("{:?}", current_state());
    }
}
