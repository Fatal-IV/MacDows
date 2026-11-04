//! The dock's right-click menu: what it offers for a pinned icon, a running one and the empty dock.
use serde::Deserialize;
use std::sync::Mutex;
use tauri::menu::{ContextMenu, IsMenuItem, Menu, MenuItem};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTarget {
    Pinned,
    Running,
    Empty,
}

impl MenuTarget {
    pub fn parse(kind: &str) -> Option<MenuTarget> {
        match kind {
            "pinned" => Some(MenuTarget::Pinned),
            "running" => Some(MenuTarget::Running),
            "empty" => Some(MenuTarget::Empty),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    NewWindow,
    Unpin,
    Pin,
    ChangeIcon,
    ResetIcon,
    Quit,
    AddApp,
    MacdowsSettings,
    TaskManager,
}

const ALL_ACTIONS: [MenuAction; 9] = [
    MenuAction::NewWindow,
    MenuAction::Unpin,
    MenuAction::Pin,
    MenuAction::ChangeIcon,
    MenuAction::ResetIcon,
    MenuAction::Quit,
    MenuAction::AddApp,
    MenuAction::MacdowsSettings,
    MenuAction::TaskManager,
];

impl MenuAction {
    pub fn id(self) -> &'static str {
        match self {
            MenuAction::NewWindow => "new_window",
            MenuAction::Unpin => "unpin",
            MenuAction::Pin => "pin",
            MenuAction::ChangeIcon => "change_icon",
            MenuAction::ResetIcon => "reset_icon",
            MenuAction::Quit => "quit",
            MenuAction::AddApp => "add_app",
            MenuAction::MacdowsSettings => "macdows_settings",
            MenuAction::TaskManager => "task_manager",
        }
    }

    pub fn from_id(id: &str) -> Option<MenuAction> {
        ALL_ACTIONS.into_iter().find(|a| a.id() == id)
    }

    /// Actions whose features exist yet; the Add app panel is built later and stays out of the menu until then.
    fn available(self) -> bool {
        !matches!(self, MenuAction::AddApp)
    }
}

/// What the menu offers, in order.
pub fn menu_plan(target: MenuTarget, running: bool, has_custom_icon: bool) -> Vec<MenuAction> {
    let mut out = match target {
        MenuTarget::Empty => return vec![MenuAction::AddApp, MenuAction::MacdowsSettings, MenuAction::TaskManager],
        MenuTarget::Pinned => vec![MenuAction::NewWindow, MenuAction::Unpin, MenuAction::ChangeIcon],
        MenuTarget::Running => vec![MenuAction::NewWindow, MenuAction::Pin, MenuAction::ChangeIcon],
    };
    if has_custom_icon {
        out.push(MenuAction::ResetIcon);
    }
    if running {
        out.push(MenuAction::Quit);
    }
    out
}

/// What the dock tells Rust about the icon that was right-clicked.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct DockMenuTarget {
    pub kind: String,
    pub id: String,
    pub exe: String,
    pub aumid: String,
    pub name: String,
    pub hwnds: Vec<isize>,
}

/// The icon the open menu belongs to; the chosen item is handled against it.
static CURRENT: Mutex<Option<DockMenuTarget>> = Mutex::new(None);
const PREFIX: &str = "dock-menu:";

/// Shows the right-click menu for the icon (or the empty dock) at the pointer.
#[tauri::command]
pub fn dock_menu(app: AppHandle, target: DockMenuTarget) -> Result<(), String> {
    let kind = MenuTarget::parse(&target.kind).ok_or("unknown menu target")?;
    let (lang, custom) = {
        let state = app.state::<crate::AppSettings>();
        let g = state.0.lock().unwrap();
        let custom = g.dock.custom_icons.keys().any(|k| k.eq_ignore_ascii_case(&target.id) || (!target.exe.is_empty() && k.eq_ignore_ascii_case(&target.exe)));
        (g.language.clone(), custom)
    };
    let actions: Vec<MenuAction> = menu_plan(kind, !target.hwnds.is_empty(), custom).into_iter().filter(|a| a.available()).collect();
    if actions.is_empty() {
        return Ok(());
    }
    let items: Vec<MenuItem<_>> = actions
        .iter()
        .map(|a| MenuItem::with_id(&app, format!("{PREFIX}{}", a.id()), crate::i18n::dock_menu_label(&lang, a.id()), true, None::<&str>))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let refs: Vec<&dyn IsMenuItem<_>> = items.iter().map(|i| i as &dyn IsMenuItem<_>).collect();
    let menu = Menu::with_items(&app, &refs).map_err(|e| e.to_string())?;
    *CURRENT.lock().unwrap() = Some(target);
    let window = app.get_webview_window("dock").ok_or("no dock window")?;
    menu.popup(window.as_ref().window().clone()).map_err(|e| e.to_string())
}

/// App-wide menu events: runs the chosen action on the icon the menu was opened for.
pub fn on_menu_event(app: &AppHandle, menu_id: &str) {
    let Some(action) = menu_id.strip_prefix(PREFIX).and_then(MenuAction::from_id) else { return };
    let Some(target) = CURRENT.lock().unwrap().clone() else { return };
    let app = app.clone();
    std::thread::spawn(move || {
        match action {
            MenuAction::NewWindow => crate::pinned_apps::launch_new(&target.id, &target.exe, &target.aumid),
            MenuAction::Unpin => crate::pinned_apps::remove_pin(&app, target.id),
            MenuAction::Pin => crate::pinned_apps::pin_app(&app, &target.exe, &target.aumid, &target.name),
            MenuAction::Quit => crate::pinned_apps::close_windows(&target.hwnds),
            MenuAction::MacdowsSettings => crate::open_settings(&app),
            MenuAction::TaskManager => {
                let _ = crate::quick_settings::quick_launch("task_manager".into());
            }
            MenuAction::ChangeIcon => change_icon(&app, &target),
            MenuAction::ResetIcon => reset_icon(&app, &target),
            MenuAction::AddApp => {}
        }
        let _ = app.emit("dock-menu-closed", ());
    });
}

/// What a custom icon is saved under: the pin's id, or for a running app that is not pinned its program.
pub fn icon_key(target: &DockMenuTarget) -> String {
    if target.id.is_empty() { target.exe.to_lowercase() } else { target.id.to_lowercase() }
}

fn change_icon(app: &AppHandle, target: &DockMenuTarget) {
    let key = icon_key(target);
    if key.is_empty() {
        return;
    }
    let Some(source) = crate::custom_icons::choose_file() else { return };
    match crate::custom_icons::save(&key, &source) {
        Ok(file) => crate::pinned_apps::update_settings(app, |s| {
            s.dock.custom_icons.insert(key.clone(), file.clone());
        }),
        Err(reason) => {
            let lang = app.state::<crate::AppSettings>().0.lock().unwrap().language.clone();
            warn(crate::i18n::icon_error(&lang, &reason));
        }
    }
}

fn reset_icon(app: &AppHandle, target: &DockMenuTarget) {
    let key = icon_key(target);
    crate::pinned_apps::update_settings(app, |s| {
        let found: Vec<String> = s.dock.custom_icons.keys().filter(|k| k.eq_ignore_ascii_case(&key)).cloned().collect();
        for k in found {
            if let Some(file) = s.dock.custom_icons.remove(&k) {
                crate::custom_icons::remove(&file);
            }
        }
    });
}

/// A small message box (the dock has no room for text).
fn warn(text: String) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONWARNING, MB_OK};
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "MacDows".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(None, PCWSTR(wide.as_ptr()), PCWSTR(title.as_ptr()), MB_OK | MB_ICONWARNING);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pinned_app_gets_new_window_unpin_and_change_icon() {
        assert_eq!(menu_plan(MenuTarget::Pinned, false, false), vec![MenuAction::NewWindow, MenuAction::Unpin, MenuAction::ChangeIcon]);
    }

    #[test]
    fn a_running_pinned_app_can_also_quit_and_a_custom_icon_can_be_reset() {
        assert_eq!(
            menu_plan(MenuTarget::Pinned, true, true),
            vec![MenuAction::NewWindow, MenuAction::Unpin, MenuAction::ChangeIcon, MenuAction::ResetIcon, MenuAction::Quit]
        );
    }

    #[test]
    fn a_running_unpinned_app_can_be_pinned() {
        assert_eq!(menu_plan(MenuTarget::Running, true, false), vec![MenuAction::NewWindow, MenuAction::Pin, MenuAction::ChangeIcon, MenuAction::Quit]);
    }

    #[test]
    fn the_empty_dock_offers_add_app_settings_and_task_manager() {
        assert_eq!(menu_plan(MenuTarget::Empty, false, false), vec![MenuAction::AddApp, MenuAction::MacdowsSettings, MenuAction::TaskManager]);
    }

    #[test]
    fn actions_round_trip_through_their_ids() {
        for a in ALL_ACTIONS {
            assert_eq!(MenuAction::from_id(a.id()), Some(a));
        }
        assert_eq!(MenuAction::from_id("nope"), None);
    }

    #[test]
    fn icons_are_saved_under_the_pin_or_for_other_apps_under_the_program() {
        let pinned = DockMenuTarget { id: "C:/Pins/Edge.lnk".into(), exe: "C:/E/msedge.exe".into(), ..Default::default() };
        assert_eq!(icon_key(&pinned), "c:/pins/edge.lnk");
        let running = DockMenuTarget { exe: "C:/Apps/App.EXE".into(), ..Default::default() };
        assert_eq!(icon_key(&running), "c:/apps/app.exe");
    }

    #[test]
    fn kinds_parse_and_unknown_kinds_do_not() {
        assert_eq!(MenuTarget::parse("pinned"), Some(MenuTarget::Pinned));
        assert_eq!(MenuTarget::parse("running"), Some(MenuTarget::Running));
        assert_eq!(MenuTarget::parse("empty"), Some(MenuTarget::Empty));
        assert_eq!(MenuTarget::parse("x"), None);
    }

    #[test]
    fn every_action_has_a_label_in_both_languages() {
        for a in ALL_ACTIONS {
            assert!(!crate::i18n::dock_menu_label("en", a.id()).is_empty(), "{}", a.id());
            assert!(!crate::i18n::dock_menu_label("tr", a.id()).is_empty(), "{}", a.id());
        }
    }
}
