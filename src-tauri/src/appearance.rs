//! Windows accent color for the "adaptive", "system accent" and Material styles.
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

/// The accent color as `#rrggbb`. Windows stores it as ABGR in the DWM key.
fn accent_from_registry() -> Option<String> {
    let dwm = RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Software\Microsoft\Windows\DWM").ok()?;
    let abgr: u32 = dwm.get_value("AccentColor").ok()?;
    Some(format!("#{:02x}{:02x}{:02x}", abgr & 0xff, (abgr >> 8) & 0xff, (abgr >> 16) & 0xff))
}

#[tauri::command]
pub fn get_system_accent_color() -> Result<String, String> {
    accent_from_registry().ok_or_else(|| "Windows accent color is not available".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn abgr_is_converted_to_rgb_hex() {
        let abgr: u32 = 0xff_d4_78_00; // A=ff B=d4 G=78 R=00 -> #0078d4
        assert_eq!(format!("#{:02x}{:02x}{:02x}", abgr & 0xff, (abgr >> 8) & 0xff, (abgr >> 16) & 0xff), "#0078d4");
    }
}
