//! Pairing new Bluetooth devices from the island: a watcher lists unpaired devices nearby, and a pairing runs on its
//! own thread, asking the page to confirm a code or to take a PIN when the device wants one. When no page is
//! listening, a pairing request is declined so nothing is left hanging.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::core::{Interface, HSTRING};
use windows::Devices::Enumeration::{
    DeviceInformation, DeviceInformationKind, DevicePairingKinds, DevicePairingProtectionLevel, DevicePairingRequestedEventArgs, DeviceWatcher,
};
use windows::Foundation::{IReference, TypedEventHandler};
use windows_collections::IIterable;

const PAIRED_KEY: &str = "System.Devices.Aep.IsPaired";
const NAME_KEY: &str = "System.ItemNameDisplay";
/// Classic Bluetooth and Bluetooth LE association endpoints.
const FILTER: &str = "(System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\" OR System.Devices.Aep.ProtocolId:=\"{bb7bb05e-5972-42b5-94fc-76eaa7084d49}\")";

/// `DevicePairingKinds` flags (ConfirmOnly = 1, DisplayPin = 2, ProvidePin = 4, ConfirmPinMatch = 8) to what the page asks the user.
/// The strongest requirement wins: a PIN to type beats a code to compare beats a plain yes.
pub fn pairing_kind(kinds: u32) -> &'static str {
    if kinds & 4 != 0 {
        "provide_pin"
    } else if kinds & (8 | 2) != 0 {
        "confirm_pin"
    } else if kinds & 1 != 0 {
        "confirm"
    } else {
        "none"
    }
}

/// Only named, not yet paired devices are worth listing: nameless ones are beacons and other noise.
pub fn visible_in_scan(name: &str, paired: bool) -> bool {
    !paired && !name.trim().is_empty()
}

#[derive(Serialize, Clone)]
struct Found {
    id: String,
    name: String,
    kind: &'static str,
}

#[derive(Serialize, Clone)]
struct Lost {
    id: String,
}

#[derive(Serialize, Clone)]
struct PairingRequest {
    id: String,
    kind: &'static str,
    pin: Option<String>,
}

static WATCHER: Mutex<Option<DeviceWatcher>> = Mutex::new(None);
/// Whether a page is open to answer pairing requests.
static LISTENING: AtomicBool = AtomicBool::new(false);
/// Where the page's answer to the open pairing request goes: (accepted, typed PIN).
static ANSWER: Mutex<Option<Sender<(bool, Option<String>)>>> = Mutex::new(None);

fn bool_prop(info: &windows_collections::IMapView<HSTRING, windows::core::IInspectable>, key: &str) -> Option<bool> {
    info.Lookup(&HSTRING::from(key)).ok()?.cast::<IReference<bool>>().ok()?.Value().ok()
}

fn name_prop(info: &windows_collections::IMapView<HSTRING, windows::core::IInspectable>) -> Option<String> {
    let value = info.Lookup(&HSTRING::from(NAME_KEY)).ok()?.cast::<IReference<HSTRING>>().ok()?.Value().ok()?;
    Some(value.to_string())
}

fn start_watcher(app: AppHandle) -> windows::core::Result<()> {
    let props: Vec<HSTRING> = vec![HSTRING::from(PAIRED_KEY), HSTRING::from(NAME_KEY)];
    let props: IIterable<HSTRING> = props.into();
    let watcher = DeviceInformation::CreateWatcherWithKindAqsFilterAndAdditionalProperties(&HSTRING::from(FILTER), &props, DeviceInformationKind::AssociationEndpoint)?;

    let a = app.clone();
    watcher.Added(&TypedEventHandler::new(move |_, info: windows::core::Ref<DeviceInformation>| {
        if let Some(info) = info.as_ref() {
            let name = info.Name().map(|n| n.to_string()).unwrap_or_default();
            let paired = info.Properties().ok().and_then(|p| bool_prop(&p, PAIRED_KEY)).unwrap_or(false);
            if visible_in_scan(&name, paired) {
                let _ = a.emit("bt-found", Found { id: info.Id()?.to_string(), name, kind: "other" });
            }
        }
        Ok(())
    }))?;
    let a = app.clone();
    watcher.Updated(&TypedEventHandler::new(move |_, update: windows::core::Ref<windows::Devices::Enumeration::DeviceInformationUpdate>| {
        if let Some(update) = update.as_ref() {
            let props = update.Properties()?;
            let paired = bool_prop(&props, PAIRED_KEY).unwrap_or(false);
            if paired {
                let _ = a.emit("bt-lost", Lost { id: update.Id()?.to_string() });
            } else if let Some(name) = name_prop(&props) {
                // A device often shows up nameless first and gets its name in an update.
                if visible_in_scan(&name, false) {
                    let _ = a.emit("bt-found", Found { id: update.Id()?.to_string(), name, kind: "other" });
                }
            }
        }
        Ok(())
    }))?;
    let a = app;
    watcher.Removed(&TypedEventHandler::new(move |_, update: windows::core::Ref<windows::Devices::Enumeration::DeviceInformationUpdate>| {
        if let Some(update) = update.as_ref() {
            let _ = a.emit("bt-lost", Lost { id: update.Id()?.to_string() });
        }
        Ok(())
    }))?;
    watcher.Start()?;
    if let Some(old) = WATCHER.lock().unwrap().replace(watcher) {
        let _ = old.Stop();
    }
    Ok(())
}

/// The page opened: list unpaired devices nearby and answer pairing requests.
#[tauri::command]
pub fn bt_scan_start(app: AppHandle) -> Result<(), String> {
    LISTENING.store(true, Ordering::SeqCst);
    start_watcher(app).map_err(|e| e.message())
}

/// The page closed (or the island hid): stop scanning and decline whatever is still being asked.
#[tauri::command]
pub fn bt_scan_stop() {
    LISTENING.store(false, Ordering::SeqCst);
    if let Some(w) = WATCHER.lock().unwrap().take() {
        let _ = w.Stop();
    }
    if let Some(tx) = ANSWER.lock().unwrap().take() {
        let _ = tx.send((false, None));
    }
}

/// The user's answer to a `bt-pairing-request`.
#[tauri::command]
pub fn bt_pairing_answer(accept: bool, pin: Option<String>) {
    if let Some(tx) = ANSWER.lock().unwrap().take() {
        let _ = tx.send((accept, pin));
    }
}

fn pair_blocking(app: &AppHandle, id: &str) -> Result<(), String> {
    let info = DeviceInformation::CreateFromIdAsync(&HSTRING::from(id)).map_err(|e| e.message())?.get().map_err(|e| e.message())?;
    let custom = info.Pairing().and_then(|p| p.Custom()).map_err(|e| e.message())?;
    let app = app.clone();
    let device_id = id.to_string();
    let token = custom
        .PairingRequested(&TypedEventHandler::new(move |_, args: windows::core::Ref<DevicePairingRequestedEventArgs>| {
            let Some(args) = args.as_ref() else { return Ok(()) };
            let deferral = args.GetDeferral()?;
            // With no page open, the request is declined by simply not accepting it.
            if LISTENING.load(Ordering::SeqCst) {
                let kind = pairing_kind(args.PairingKind()?.0 as u32);
                let pin = args.Pin().ok().map(|p| p.to_string()).filter(|p| !p.is_empty());
                let (tx, rx) = channel();
                *ANSWER.lock().unwrap() = Some(tx);
                let _ = app.emit("bt-pairing-request", PairingRequest { id: device_id.clone(), kind, pin });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok((true, typed)) => match (kind, typed) {
                        ("provide_pin", Some(pin)) => args.AcceptWithPin(&HSTRING::from(pin))?,
                        ("provide_pin", None) => {}
                        _ => args.Accept()?,
                    },
                    _ => {}
                }
                ANSWER.lock().unwrap().take();
            }
            deferral.Complete()
        }))
        .map_err(|e| e.message())?;
    let kinds = DevicePairingKinds(1 | 2 | 4 | 8);
    let result = custom.PairWithProtectionLevelAsync(kinds, DevicePairingProtectionLevel::Default).and_then(|op| op.get());
    let _ = custom.RemovePairingRequested(token);
    let result = result.map_err(|e| e.message())?;
    // Paired = 0, AlreadyPaired = 3.
    match result.Status().map(|s| s.0) {
        Ok(0) | Ok(3) => Ok(()),
        Ok(7) | Ok(18) | Ok(19) => Err("rejected".into()),
        _ => Err("failed".into()),
    }
}

/// `DeviceUnpairingResultStatus`: Unpaired = 0, AlreadyUnpaired = 1.
pub fn unpair_ok(status: i32) -> bool {
    matches!(status, 0 | 1)
}

/// Removes the pairing of a device from the paired list.
#[tauri::command]
pub async fn bt_unpair(id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let info = DeviceInformation::CreateFromIdAsync(&HSTRING::from(id.as_str())).map_err(|e| e.message())?.get().map_err(|e| e.message())?;
        let result = info.Pairing().and_then(|p| p.UnpairAsync()).and_then(|op| op.get()).map_err(|e| e.message())?;
        if unpair_ok(result.Status().map_err(|e| e.message())?.0) { Ok(()) } else { Err("failed".to_string()) }
    })
    .await
    .map_err(|_| "failed".to_string())?
}

/// Pairs the device the user picked. Runs on its own thread; the page also gets `bt-paired` when it is done.
#[tauri::command]
pub async fn bt_pair(app: AppHandle, id: String) -> Result<(), String> {
    let handle = app.clone();
    let device = id.clone();
    let result = tauri::async_runtime::spawn_blocking(move || pair_blocking(&handle, &device)).await.map_err(|_| "failed".to_string())?;
    if result.is_ok() {
        let _ = app.emit("bt-paired", Lost { id });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairing_kinds_map_to_the_strongest_requirement() {
        assert_eq!(pairing_kind(0), "none");
        assert_eq!(pairing_kind(1), "confirm");
        assert_eq!(pairing_kind(2), "confirm_pin");
        assert_eq!(pairing_kind(4), "provide_pin");
        assert_eq!(pairing_kind(8), "confirm_pin");
        assert_eq!(pairing_kind(1 | 8), "confirm_pin");
        assert_eq!(pairing_kind(1 | 2 | 4 | 8), "provide_pin");
        assert_eq!(pairing_kind(1 | 4), "provide_pin");
    }

    #[test]
    fn unpairing_counts_unpaired_and_already_unpaired_as_done() {
        assert!(unpair_ok(0));
        assert!(unpair_ok(1));
        assert!(!unpair_ok(2));
        assert!(!unpair_ok(3));
        assert!(!unpair_ok(4));
    }

    #[test]
    fn only_named_unpaired_devices_are_listed() {
        assert!(!visible_in_scan("", false));
        assert!(!visible_in_scan("   ", false));
        assert!(!visible_in_scan("Headphones", true));
        assert!(visible_in_scan("Headphones", false));
    }
}
