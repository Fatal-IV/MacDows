//! Paired Bluetooth devices for the notch: who is connected, their battery level, and connect / disconnect.
//!
//! Connecting uses the Bluetooth service state of the audio profiles (A2DP sink, hands-free): switching them
//! off and on makes Windows reconnect, switching them off disconnects. That works for headphones and speakers;
//! other kinds of device (mice, keyboards, some LE gadgets) are left to Windows's own settings.

use serde::Serialize;
use std::collections::HashMap;
use windows::core::{Interface, GUID, HSTRING, PCWSTR};
use windows::Devices::Bluetooth::{BluetoothConnectionStatus, BluetoothDevice, BluetoothMajorClass};
use windows::Devices::Enumeration::Pnp::{PnpObject, PnpObjectType};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Foundation::IReference;
use windows_collections::IIterable;
use windows::Win32::Devices::Bluetooth::{
    BluetoothFindFirstRadio, BluetoothFindRadioClose, BluetoothGetDeviceInfo, BluetoothSetServiceState, BLUETOOTH_ADDRESS, BLUETOOTH_DEVICE_INFO,
    BLUETOOTH_FIND_RADIO_PARAMS,
};
use windows::Win32::Foundation::{CloseHandle, HANDLE};

/// `System.Devices.BatteryLifePercent`-style key Windows fills for Bluetooth devices that report a battery.
const BATTERY_KEY: &str = "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2";

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct BtDevice {
    pub id: String,
    pub name: String,
    /// The 48-bit address as 12 hex digits.
    pub address: String,
    pub connected: bool,
    pub battery: Option<u8>,
    /// "audio", "phone", "computer", "input" or "other".
    pub kind: String,
}

pub fn address_hex(address: u64) -> String {
    format!("{:012X}", address & 0xFFFF_FFFF_FFFF)
}

pub fn kind_of(major: BluetoothMajorClass) -> &'static str {
    match major {
        BluetoothMajorClass::AudioVideo => "audio",
        BluetoothMajorClass::Phone => "phone",
        BluetoothMajorClass::Computer => "computer",
        BluetoothMajorClass::Peripheral => "input",
        _ => "other",
    }
}

/// A battery value Windows reports as a percentage; anything above 100 is not one.
pub fn battery_percent(raw: u32) -> Option<u8> {
    (raw <= 100).then_some(raw as u8)
}

/// The profiles switched to connect or disconnect a device: A2DP sink and hands-free.
const SERVICES: [GUID; 2] = [GUID::from_u128(0x0000110B_0000_1000_8000_00805F9B34FB), GUID::from_u128(0x0000111E_0000_1000_8000_00805F9B34FB)];

/// Battery levels by device address, read from the Windows device nodes whose id contains the address.
fn batteries(addresses: &[String]) -> HashMap<String, u8> {
    let mut out = HashMap::new();
    let key = HSTRING::from(BATTERY_KEY);
    let props: IIterable<HSTRING> = vec![key.clone()].into();
    let Ok(nodes) = PnpObject::FindAllAsync(PnpObjectType::Device, &props).and_then(|op| op.get()) else { return out };
    for node in nodes {
        let Ok(id) = node.Id() else { continue };
        let id = id.to_string().to_ascii_uppercase();
        let Some(address) = addresses.iter().find(|a| id.contains(a.as_str())) else { continue };
        let Ok(map) = node.Properties() else { continue };
        let Ok(value) = map.Lookup(&key) else { continue };
        let raw = value
            .cast::<IReference<u8>>()
            .and_then(|r| r.Value())
            .map(u32::from)
            .or_else(|_| value.cast::<IReference<u32>>().and_then(|r| r.Value()))
            .or_else(|_| value.cast::<IReference<i32>>().and_then(|r| r.Value()).map(|v| v.max(0) as u32));
        if let Some(p) = raw.ok().and_then(battery_percent) {
            out.insert(address.clone(), p);
        }
    }
    out
}

fn read_devices() -> Vec<BtDevice> {
    let Ok(selector) = BluetoothDevice::GetDeviceSelectorFromPairingState(true) else { return Vec::new() };
    let Ok(found) = DeviceInformation::FindAllAsyncAqsFilter(&selector).and_then(|op| op.get()) else { return Vec::new() };
    let mut list: Vec<BtDevice> = Vec::new();
    for info in found {
        let Ok(id) = info.Id() else { continue };
        let Ok(device) = BluetoothDevice::FromIdAsync(&id).and_then(|op| op.get()) else { continue };
        let address = address_hex(device.BluetoothAddress().unwrap_or(0));
        let connected = device.ConnectionStatus().ok() == Some(BluetoothConnectionStatus::Connected);
        let kind = device.ClassOfDevice().and_then(|c| c.MajorClass()).map(kind_of).unwrap_or("other");
        list.push(BtDevice { id: id.to_string(), name: info.Name().map(|n| n.to_string()).unwrap_or_default(), address, connected, battery: None, kind: kind.into() });
    }
    let addresses: Vec<String> = list.iter().map(|d| d.address.clone()).collect();
    let levels = batteries(&addresses);
    for d in &mut list {
        d.battery = levels.get(&d.address).copied();
    }
    // Connected first, then by name.
    list.sort_by(|a, b| b.connected.cmp(&a.connected).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    list
}

#[tauri::command]
pub async fn get_bluetooth_devices() -> Vec<BtDevice> {
    tauri::async_runtime::spawn_blocking(read_devices).await.unwrap_or_default()
}

fn radio() -> Option<HANDLE> {
    let params = BLUETOOTH_FIND_RADIO_PARAMS { dwSize: std::mem::size_of::<BLUETOOTH_FIND_RADIO_PARAMS>() as u32 };
    let mut handle = HANDLE::default();
    unsafe {
        let find = BluetoothFindFirstRadio(&params, &mut handle).ok()?;
        let _ = BluetoothFindRadioClose(find);
    }
    Some(handle)
}

fn set_services(address: u64, enable: bool) -> Result<(), String> {
    let radio = radio().ok_or("no Bluetooth radio")?;
    let mut info = BLUETOOTH_DEVICE_INFO { dwSize: std::mem::size_of::<BLUETOOTH_DEVICE_INFO>() as u32, Address: BLUETOOTH_ADDRESS { Anonymous: windows::Win32::Devices::Bluetooth::BLUETOOTH_ADDRESS_0 { ullLong: address } }, ..Default::default() };
    let result = unsafe {
        if BluetoothGetDeviceInfo(Some(radio), &mut info) != 0 {
            Err("device not found".to_string())
        } else {
            for service in SERVICES.iter() {
                // The state is switched off first so that "connect" also works when Windows thinks it is already on.
                let _ = BluetoothSetServiceState(Some(radio), &info, service, 0);
                if enable {
                    let _ = BluetoothSetServiceState(Some(radio), &info, service, 1);
                }
            }
            Ok(())
        }
    };
    unsafe {
        let _ = CloseHandle(radio);
    }
    let _ = PCWSTR::null();
    result
}

#[tauri::command]
pub async fn set_bluetooth_connection(address: String, connect: bool) -> Result<Vec<BtDevice>, String> {
    let parsed = u64::from_str_radix(&address, 16).map_err(|_| "bad address".to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        set_services(parsed, connect)?;
        // Give Windows a moment to finish (re)connecting before the list is read again.
        std::thread::sleep(std::time::Duration::from_millis(if connect { 2500 } else { 800 }));
        Ok(read_devices())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_is_twelve_hex_digits() {
        assert_eq!(address_hex(0x5CD33D181BFE), "5CD33D181BFE");
        assert_eq!(address_hex(0x1234), "000000001234");
    }

    #[test]
    fn device_classes_map_to_a_kind() {
        assert_eq!(kind_of(BluetoothMajorClass::AudioVideo), "audio");
        assert_eq!(kind_of(BluetoothMajorClass::Phone), "phone");
        assert_eq!(kind_of(BluetoothMajorClass::Peripheral), "input");
        assert_eq!(kind_of(BluetoothMajorClass::Imaging), "other");
    }

    #[test]
    fn only_percentages_count_as_battery() {
        assert_eq!(battery_percent(88), Some(88));
        assert_eq!(battery_percent(100), Some(100));
        assert_eq!(battery_percent(255), None);
    }
}
