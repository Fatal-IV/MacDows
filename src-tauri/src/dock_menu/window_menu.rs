//! A running app's right-click menu: the dock's own items, a separator, then the window's own Windows menu
//! (Restore, Move, Size, Minimize, Maximize, Close), taken from `GetSystemMenu` so Windows words and enables it.
//! The taskbar itself is never touched.
use windows::core::PWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, GetMenuItemCount, GetMenuItemInfoW, GetSystemMenu, PostMessageW, SendMessageTimeoutW,
    SetForegroundWindow, TrackPopupMenuEx, HMENU, MENUITEMINFOW, MFS_DISABLED, MFS_GRAYED, MFT_SEPARATOR, MF_GRAYED, MF_SEPARATOR, MF_STRING,
    MIIM_FTYPE, MIIM_ID, MIIM_STATE, MIIM_STRING, SMTO_ABORTIFHUNG, TPM_BOTTOMALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_INITMENUPOPUP, WM_NULL,
    WM_SYSCOMMAND,
};

/// Window-menu commands (`SC_*`) all live at 0xF000 and above; the dock's own items use 1..=n.
const SYSTEM_BASE: u32 = 0xF000;

#[derive(Debug, PartialEq, Eq)]
pub enum Choice {
    /// Index into the dock's own items.
    Dock(usize),
    /// A `SC_*` command for the window.
    System(u32),
    None,
}

/// Maps the id `TrackPopupMenuEx` returned (0 = dismissed) to what was chosen.
pub fn classify(id: u32, dock_items: usize) -> Choice {
    match id {
        0 => Choice::None,
        n if n >= SYSTEM_BASE => Choice::System(n),
        n if (n as usize) <= dock_items => Choice::Dock(n as usize - 1),
        _ => Choice::None,
    }
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Copies the window's system menu items (after letting the window update their enabled state).
unsafe fn append_system_items(menu: HMENU, hwnd: HWND) {
    let sys = unsafe { GetSystemMenu(hwnd, false) };
    if sys.is_invalid() {
        return;
    }
    // Windows fixes Restore / Move / Size / Maximize greying when the menu is about to show: ask for that.
    // lparam: low word = position 0, high word = 1 marks a window menu.
    unsafe { SendMessageTimeoutW(hwnd, WM_INITMENUPOPUP, WPARAM(sys.0 as usize), LPARAM(1 << 16), SMTO_ABORTIFHUNG, 200, None) };
    let count = unsafe { GetMenuItemCount(Some(sys)) };
    let mut first = true;
    for i in 0..count.max(0) as u32 {
        let mut info = MENUITEMINFOW { cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32, fMask: MIIM_ID | MIIM_FTYPE | MIIM_STATE | MIIM_STRING, ..Default::default() };
        if unsafe { GetMenuItemInfoW(sys, i, true, &mut info) }.is_err() {
            continue;
        }
        if info.fType.0 & MFT_SEPARATOR.0 != 0 {
            if !first {
                let _ = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, None) };
            }
            continue;
        }
        // Items the app added itself carry small ids that would clash with the dock's own.
        if info.wID < SYSTEM_BASE {
            continue;
        }
        let mut buf = vec![0u16; info.cch as usize + 1];
        info.dwTypeData = PWSTR(buf.as_mut_ptr());
        info.cch += 1;
        if unsafe { GetMenuItemInfoW(sys, i, true, &mut info) }.is_err() {
            continue;
        }
        let grayed = info.fState.0 & (MFS_GRAYED.0 | MFS_DISABLED.0) != 0;
        let flags = if grayed { MF_STRING | MF_GRAYED } else { MF_STRING };
        let _ = unsafe { AppendMenuW(menu, flags, info.wID as usize, windows::core::PCWSTR(buf.as_ptr())) };
        first = false;
    }
}

/// Shows the menu at the pointer and returns the id of the chosen item (0 when dismissed).
pub fn popup(owner: HWND, target: HWND, labels: &[String]) -> u32 {
    unsafe {
        let Ok(menu) = CreatePopupMenu() else { return 0 };
        for (i, label) in labels.iter().enumerate() {
            let text = wide(label);
            let _ = AppendMenuW(menu, MF_STRING, i + 1, windows::core::PCWSTR(text.as_ptr()));
        }
        if !labels.is_empty() {
            let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        }
        append_system_items(menu, target);
        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);
        // Without the owner in front, the menu would not close when the user clicks elsewhere.
        let _ = SetForegroundWindow(owner);
        let id = TrackPopupMenuEx(menu, (TPM_RETURNCMD | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN).0, pt.x, pt.y, owner, None).0 as u32;
        let _ = PostMessageW(Some(owner), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
        id
    }
}

pub fn send_system_command(hwnd: HWND, id: u32) {
    unsafe {
        let _ = PostMessageW(Some(hwnd), WM_SYSCOMMAND, WPARAM(id as usize), LPARAM(0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_map_to_dock_items_system_commands_or_nothing() {
        assert_eq!(classify(0, 3), Choice::None);
        assert_eq!(classify(1, 3), Choice::Dock(0));
        assert_eq!(classify(3, 3), Choice::Dock(2));
        assert_eq!(classify(4, 3), Choice::None);
        assert_eq!(classify(0xF060, 3), Choice::System(0xF060));
        assert_eq!(classify(0xF120, 0), Choice::System(0xF120));
    }
}
