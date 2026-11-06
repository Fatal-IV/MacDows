//! A device, network or VPN connected or disconnected: the island shows its name with an animated icon.
//!
//! Bluetooth devices, the Wi-Fi network and VPN connections are looked at every few seconds. What is already
//! connected when MacDows starts is not announced; only changes afterwards send a `link-changed` event
//! (`kind` is "bluetooth", "wifi" or "vpn", `state` is "connected" or "disconnected").

use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Link {
    pub kind: &'static str,
    pub name: String,
    pub state: &'static str,
}

/// What is connected: a stable key (address, SSID, entry name) and the name to show.
type Connected = BTreeMap<String, String>;

/// Names that are in `now` but not in `before` (new), and the other way round (gone).
pub fn changes(before: &Connected, now: &Connected) -> (Vec<String>, Vec<String>) {
    let added = now.iter().filter(|(k, _)| !before.contains_key(*k)).map(|(_, n)| n.clone()).collect();
    let removed = before.iter().filter(|(k, _)| !now.contains_key(*k)).map(|(_, n)| n.clone()).collect();
    (added, removed)
}

fn named(name: String, fallback: &str) -> String {
    if name.trim().is_empty() { fallback.to_string() } else { name }
}

struct Watch {
    kind: &'static str,
    fallback: &'static str,
    before: Option<Connected>,
}

impl Watch {
    fn new(kind: &'static str, fallback: &'static str) -> Self {
        Watch { kind, fallback, before: None }
    }

    /// Compares with the last reading and tells the island. The first reading only sets the baseline; `None` (the
    /// source could not be read) keeps what was known.
    fn feed(&mut self, app: &AppHandle, now: Option<Connected>) {
        let Some(now) = now else { return };
        if let Some(before) = &self.before {
            let (added, removed) = changes(before, &now);
            for name in &added {
                let _ = app.emit("link-changed", Link { kind: self.kind, name: named(name.clone(), self.fallback), state: "connected" });
            }
            // Moving from one Wi-Fi network to another is one connect, not a disconnect plus a connect.
            if !(self.kind == "wifi" && !added.is_empty()) {
                for name in removed {
                    let _ = app.emit("link-changed", Link { kind: self.kind, name: named(name, self.fallback), state: "disconnected" });
                }
            }
        }
        self.before = Some(now);
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut bluetooth = Watch::new("bluetooth", "Bluetooth");
        let mut wifi = Watch::new("wifi", "Wi-Fi");
        let mut vpn = Watch::new("vpn", "VPN");
        loop {
            std::thread::sleep(Duration::from_secs(3));

            bluetooth.feed(&app, Some(crate::bluetooth::connected_devices().into_iter().collect()));

            let state = crate::wifi::current_state();
            wifi.feed(&app, state.available.then(|| state.networks.iter().filter(|n| n.connected).map(|n| (n.ssid.clone(), n.ssid.clone())).collect()));

            let state = crate::vpn::current_state();
            let mut up: Connected = state.entries.iter().filter(|e| e.state == "connected").map(|e| (e.name.clone(), e.name.clone())).collect();
            if state.app_vpn_active {
                up.insert("\u{0}app".to_string(), "VPN".to_string());
            }
            vpn.feed(&app, Some(up));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[(&str, &str)]) -> Connected {
        items.iter().map(|(k, n)| (k.to_string(), n.to_string())).collect()
    }

    #[test]
    fn new_and_gone_entries_are_told_apart() {
        let before = set(&[("A", "Buds")]);
        let now = set(&[("B", "Mouse")]);
        assert_eq!(changes(&before, &now), (vec!["Mouse".to_string()], vec!["Buds".to_string()]));
        assert_eq!(changes(&now, &now), (vec![], vec![]));
        assert_eq!(changes(&Connected::new(), &now).0.len(), 1);
    }

    #[test]
    fn an_unnamed_device_gets_a_fallback_name() {
        assert_eq!(named("  ".into(), "Bluetooth"), "Bluetooth");
        assert_eq!(named("Buds".into(), "Bluetooth"), "Buds");
    }
}
