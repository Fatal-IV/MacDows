//! A device or network just connected: the island shows its name with an animated icon.
//!
//! Bluetooth devices and the Wi-Fi network are looked at every few seconds. What is already connected when MacDows
//! starts is not announced; only what connects afterwards sends a `link-connected` event (`kind` is "bluetooth" or "wifi").

use serde::Serialize;
use std::collections::HashSet;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Link {
    pub kind: &'static str,
    pub name: String,
}

/// The entries of `now` whose key was not connected before.
pub fn newly_connected(before: &HashSet<String>, now: &[(String, String)]) -> Vec<String> {
    now.iter().filter(|(key, _)| !before.contains(key)).map(|(_, name)| name.clone()).collect()
}

fn named(name: String, fallback: &str) -> String {
    if name.trim().is_empty() { fallback.to_string() } else { name }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut bluetooth: Option<HashSet<String>> = None;
        let mut wifi: Option<HashSet<String>> = None;
        loop {
            std::thread::sleep(Duration::from_secs(3));

            let now = crate::bluetooth::connected_devices();
            let keys: HashSet<String> = now.iter().map(|(k, _)| k.clone()).collect();
            if let Some(before) = &bluetooth {
                for name in newly_connected(before, &now) {
                    let _ = app.emit("link-connected", Link { kind: "bluetooth", name: named(name, "Bluetooth") });
                }
            }
            bluetooth = Some(keys);

            let state = crate::wifi::current_state();
            let now: Vec<(String, String)> = state.networks.iter().filter(|n| n.connected).map(|n| (n.ssid.clone(), n.ssid.clone())).collect();
            let keys: HashSet<String> = now.iter().map(|(k, _)| k.clone()).collect();
            if state.available {
                if let Some(before) = &wifi {
                    for name in newly_connected(before, &now) {
                        let _ = app.emit("link-connected", Link { kind: "wifi", name: named(name, "Wi-Fi") });
                    }
                }
                wifi = Some(keys);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(keys: &[&str]) -> HashSet<String> {
        keys.iter().map(|k| k.to_string()).collect()
    }

    #[test]
    fn only_what_was_not_connected_before_is_new() {
        let now = vec![("A".to_string(), "Buds".to_string()), ("B".to_string(), "Mouse".to_string())];
        assert_eq!(newly_connected(&set(&["A"]), &now), vec!["Mouse".to_string()]);
        assert!(newly_connected(&set(&["A", "B"]), &now).is_empty());
        assert_eq!(newly_connected(&set(&[]), &now).len(), 2);
    }

    #[test]
    fn an_unnamed_device_gets_a_fallback_name() {
        assert_eq!(named("  ".into(), "Bluetooth"), "Bluetooth");
        assert_eq!(named("Buds".into(), "Bluetooth"), "Buds");
    }
}
