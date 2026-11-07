//! Dynamic Island settings: which screens the island has, what each one offers, how it is navigated.
//! Every struct fills missing keys with its defaults, so an older settings file keeps working, and
//! `Island::normalize` keeps the values inside what the island can handle.
use crate::settings::Visibility;
use serde::{Deserialize, Serialize};

/// The island's screens, in their default order.
pub const SCREEN_IDS: [&str; 6] = ["status", "music", "tools", "quick", "notifications", "shelf"];

/// What the island does in a fullscreen app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FullscreenMode {
    /// Appears when the pointer reaches the screen edge (how it always worked).
    Edge,
    /// Never shown in a fullscreen app.
    Hidden,
}

/// How the island sits on the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IslandStyle {
    /// Stuck to the top edge of the screen (how it always was).
    #[default]
    Notch,
    /// A pill that floats a little below the top edge.
    Dynamic,
}

/// What a click on the island opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClickMode {
    /// The player while media plays, otherwise the quick settings.
    Smart,
    Quick,
    Player,
}

/// Status widgets that can sit beside the clock.
pub const WIDGET_IDS: [&str; 6] = ["weather", "battery", "cpu", "ram", "disk", "net"];
/// How many widgets fit on one side of the clock.
pub const MAX_WIDGETS_PER_SIDE: usize = 3;

/// Which widgets sit left and right of the clock, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Widgets {
    pub left: Vec<String>,
    pub right: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusCfg {
    pub enabled: bool,
    /// 12-hour clock instead of 24-hour.
    pub clock_12h: bool,
    pub widgets: Widgets,
    /// Empty: no weather request is ever made.
    pub weather_city: String,
    pub weather_lat: f64,
    pub weather_lon: f64,
    /// "celsius" or "fahrenheit".
    pub temp_unit: String,
    /// The battery widget turns red below this percent, 5 to 50 in steps of 5.
    pub low_battery: u8,
    /// Only read from settings files written before widgets existed (see `normalize`); never written back.
    #[serde(skip_serializing)]
    pub battery_percent: Option<bool>,
}

/// How many gradient effects the player has (`MusicCfg::gradient` is 0 for none, or 1 to this).
pub const GRADIENT_EFFECTS: u8 = 8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MusicCfg {
    pub enabled: bool,
    pub visualizer: bool,
    pub album_glow: bool,
    pub progress: bool,
    /// The music pill comes up by itself on a new track.
    pub auto_open: bool,
    pub artwork: bool,
    /// Buttons to switch between programs that play media (shown from two programs).
    pub sources: bool,
    /// The moving gradient behind the player and the small island: 0 is off, 1 to 8 pick one of the eight effects
    /// (see `GRADIENT_FX` in `DynamicIsland.tsx`).
    pub gradient: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolsCfg {
    pub enabled: bool,
    pub calendar: bool,
    /// The week starts on Monday (otherwise Sunday).
    pub week_starts_monday: bool,
    pub timer: bool,
    /// A chime when the timer reaches zero.
    pub timer_sound: bool,
    /// A bar that shrinks with the countdown.
    pub timer_progress: bool,
    pub stopwatch: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct QuickCfg {
    pub enabled: bool,
    pub wifi: bool,
    pub bluetooth: bool,
    pub dnd: bool,
    pub battery_saver: bool,
    pub airplane: bool,
    pub hotspot: bool,
    pub accessibility: bool,
    pub brightness: bool,
    pub volume: bool,
    pub lock: bool,
    pub macdows_settings: bool,
    pub windows_settings: bool,
    /// The Wi-Fi page behind the Wi-Fi tile's arrow.
    pub wifi_page: bool,
    /// The VPN card and page.
    pub vpn: bool,
    /// Pairing new Bluetooth devices inside the island (off: "Add device" opens Windows settings).
    pub bluetooth_pairing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct NotifCfg {
    pub enabled: bool,
    /// The banner that shows a new notification.
    pub banner: bool,
    /// How long the banner stays, 3 to 10 seconds.
    pub banner_seconds: u32,
    /// Keep a history of notifications.
    pub save_history: bool,
    /// How many are kept: 25, 50 or 100.
    pub history_max: usize,
    /// Apps whose notifications get no banner (they are still recorded).
    pub muted_apps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShelfCfg {
    pub enabled: bool,
    /// Keep the shelf across logons (otherwise it empties on a new Windows session).
    pub persist: bool,
    /// How many items it holds, 1 to 100.
    pub max_items: usize,
    /// Open the shelf when files are dropped on the island.
    pub auto_open: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Screens {
    pub status: StatusCfg,
    pub music: MusicCfg,
    pub tools: ToolsCfg,
    pub quick: QuickCfg,
    pub notifications: NotifCfg,
    pub shelf: ShelfCfg,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Nav {
    /// The mouse wheel walks through the screens.
    pub wheel: bool,
    /// The order of the screens (every id once; see `SCREEN_IDS`).
    pub order: Vec<String>,
    pub click: ClickMode,
    /// The middle button opens calendar and timer.
    pub middle: bool,
    /// Coming back after the pointer left returns to the last screen.
    pub remember: bool,
    /// While media exists, hovering the island opens the player by itself.
    pub hover_player: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Island {
    pub style: IslandStyle,
    pub visibility: Visibility,
    pub fullscreen: FullscreenMode,
    pub nav: Nav,
    pub screens: Screens,
}

impl Default for StatusCfg {
    fn default() -> Self {
        StatusCfg {
            enabled: true,
            clock_12h: false,
            widgets: Widgets::default(),
            weather_city: String::new(),
            weather_lat: 0.0,
            weather_lon: 0.0,
            temp_unit: "celsius".into(),
            low_battery: 20,
            battery_percent: None,
        }
    }
}
impl Default for Widgets {
    fn default() -> Self {
        Widgets { left: vec!["weather".into()], right: vec!["battery".into()] }
    }
}
impl Default for MusicCfg {
    fn default() -> Self {
        MusicCfg { enabled: true, visualizer: true, album_glow: true, progress: true, auto_open: true, artwork: true, sources: true, gradient: 1 }
    }
}
impl Default for ToolsCfg {
    fn default() -> Self {
        ToolsCfg { enabled: true, calendar: true, week_starts_monday: true, timer: true, timer_sound: true, timer_progress: true, stopwatch: true }
    }
}
impl Default for QuickCfg {
    fn default() -> Self {
        QuickCfg { enabled: true, wifi: true, bluetooth: true, dnd: true, battery_saver: true, airplane: true, hotspot: true, accessibility: true, brightness: true, volume: true, lock: true, macdows_settings: true, windows_settings: true, wifi_page: true, vpn: true, bluetooth_pairing: true }
    }
}
impl Default for NotifCfg {
    fn default() -> Self {
        NotifCfg { enabled: true, banner: true, banner_seconds: 5, save_history: true, history_max: 100, muted_apps: Vec::new() }
    }
}
impl Default for ShelfCfg {
    fn default() -> Self {
        ShelfCfg { enabled: true, persist: false, max_items: 40, auto_open: true }
    }
}
impl Default for Nav {
    fn default() -> Self {
        Nav { wheel: true, order: SCREEN_IDS.iter().map(|s| s.to_string()).collect(), click: ClickMode::Smart, middle: true, remember: false, hover_player: true }
    }
}
impl Default for Island {
    fn default() -> Self {
        Island { style: IslandStyle::Notch, visibility: Visibility::Fixed, fullscreen: FullscreenMode::Edge, nav: Nav::default(), screens: Screens::default() }
    }
}

const HISTORY_CHOICES: [usize; 3] = [25, 50, 100];

/// `order` with unknown and repeated ids dropped and the missing ones added in their default place.
fn repaired_order(order: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in order {
        let id = id.to_ascii_lowercase();
        if SCREEN_IDS.contains(&id.as_str()) && !out.contains(&id) {
            out.push(id);
        }
    }
    for id in SCREEN_IDS {
        if !out.iter().any(|o| o == id) {
            out.push(id.to_string());
        }
    }
    out
}

/// Unknown ids dropped, each widget kept once (first place wins, left before right), at most
/// `MAX_WIDGETS_PER_SIDE` per side.
fn repaired_widgets(w: &Widgets) -> Widgets {
    let mut seen: Vec<String> = Vec::new();
    let mut side = |ids: &[String]| -> Vec<String> {
        let mut out = Vec::new();
        for id in ids {
            let id = id.to_ascii_lowercase();
            if WIDGET_IDS.contains(&id.as_str()) && !seen.contains(&id) && out.len() < MAX_WIDGETS_PER_SIDE {
                seen.push(id.clone());
                out.push(id);
            }
        }
        out
    };
    let left = side(&w.left);
    let right = side(&w.right);
    Widgets { left, right }
}

fn nearest(choices: &[usize], value: usize) -> usize {
    *choices.iter().min_by_key(|c| c.abs_diff(value)).unwrap_or(&value)
}

impl Island {
    /// Which screens are on, in `SCREEN_IDS` order.
    pub fn enabled_flags(&self) -> [bool; 6] {
        let s = &self.screens;
        [s.status.enabled, s.music.enabled, s.tools.enabled, s.quick.enabled, s.notifications.enabled, s.shelf.enabled]
    }

    /// Keeps the values inside what the island supports: at least one screen on (Status comes back if all
    /// were switched off), a full screen order, and numbers within their ranges.
    pub fn normalize(&mut self) {
        if !self.enabled_flags().iter().any(|on| *on) {
            self.screens.status.enabled = true;
        }
        self.nav.order = repaired_order(&self.nav.order);
        let st = &mut self.screens.status;
        // Files from before widgets: `battery_percent: false` meant no battery next to the clock.
        if st.battery_percent.take() == Some(false) {
            st.widgets.left.retain(|w| w != "battery");
            st.widgets.right.retain(|w| w != "battery");
        }
        st.widgets = repaired_widgets(&st.widgets);
        st.low_battery = (((st.low_battery as f32 / 5.0).round() as u8) * 5).clamp(5, 50);
        if st.temp_unit != "fahrenheit" {
            st.temp_unit = "celsius".into();
        }
        let n = &mut self.screens.notifications;
        n.banner_seconds = n.banner_seconds.clamp(3, 10);
        n.history_max = nearest(&HISTORY_CHOICES, n.history_max);
        self.screens.shelf.max_items = self.screens.shelf.max_items.clamp(1, 100);
        self.screens.music.gradient = self.screens.music.gradient.min(GRADIENT_EFFECTS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_spec() {
        let i = Island::default();
        assert_eq!(i.visibility, Visibility::Fixed);
        assert_eq!(i.fullscreen, FullscreenMode::Edge);
        assert_eq!(i.style, IslandStyle::Notch);
        assert!(i.enabled_flags().iter().all(|on| *on));
        assert!(!i.screens.status.clock_12h);
        assert!(i.screens.music.sources);
        assert_eq!(i.screens.music.gradient, 1);
        assert!(i.screens.tools.week_starts_monday && i.screens.tools.timer_sound && i.screens.tools.timer_progress);
        assert_eq!((i.screens.notifications.banner_seconds, i.screens.notifications.history_max), (5, 100));
        assert!(!i.screens.shelf.persist);
        assert_eq!(i.screens.shelf.max_items, 40);
        assert!(i.nav.wheel && i.nav.middle && !i.nav.remember && i.nav.hover_player);
        assert_eq!(i.nav.click, ClickMode::Smart);
        assert_eq!(i.nav.order, SCREEN_IDS.map(String::from).to_vec());
    }

    #[test]
    fn the_island_style_is_read_and_defaults_to_the_notch() {
        let floating: Island = serde_json::from_str(r#"{"style":"dynamic"}"#).unwrap();
        assert_eq!(floating.style, IslandStyle::Dynamic);
        let notch: Island = serde_json::from_str(r#"{"style":"notch"}"#).unwrap();
        assert_eq!(notch.style, IslandStyle::Notch);
        let old: Island = serde_json::from_str(r#"{"visibility":"fixed"}"#).unwrap();
        assert_eq!(old.style, IslandStyle::Notch, "files from before the setting stay on the notch");
        assert_eq!(serde_json::to_string(&IslandStyle::Dynamic).unwrap(), r#""dynamic""#);
    }

    #[test]
    fn the_gradient_choice_is_kept_inside_its_range() {
        for (given, kept) in [(0u8, 0u8), (5, 5), (8, 8), (9, 8), (255, 8)] {
            let mut i: Island = serde_json::from_str(&format!(r#"{{"screens":{{"music":{{"gradient":{given}}}}}}}"#)).unwrap();
            i.normalize();
            assert_eq!(i.screens.music.gradient, kept, "given {given}");
        }
        let old: Island = serde_json::from_str(r#"{"screens":{"music":{"sources":false}}}"#).unwrap();
        assert_eq!(old.screens.music.gradient, 1, "files from before the setting get the default effect");
    }

    fn status_from(json: &str) -> StatusCfg {
        let mut i: Island = serde_json::from_str(&format!(r#"{{"screens":{{"status":{json}}}}}"#)).unwrap();
        i.normalize();
        i.screens.status
    }

    #[test]
    fn status_defaults_have_weather_left_and_battery_right() {
        let s = StatusCfg::default();
        assert_eq!((s.widgets.left, s.widgets.right), (vec!["weather".to_string()], vec!["battery".to_string()]));
        assert_eq!((s.weather_city.as_str(), s.temp_unit.as_str(), s.low_battery), ("", "celsius", 20));
    }

    #[test]
    fn old_battery_percent_false_removes_the_battery_widget() {
        let s = status_from(r#"{"battery_percent":false}"#);
        assert_eq!(s.widgets.left, ["weather"]);
        assert!(s.widgets.right.is_empty());
    }

    #[test]
    fn old_battery_percent_true_keeps_the_default_layout() {
        assert_eq!(status_from(r#"{"battery_percent":true}"#).widgets, Widgets::default());
    }

    #[test]
    fn unknown_duplicate_and_excess_widgets_are_repaired() {
        let s = status_from(r#"{"widgets":{"left":["cpu","bogus","cpu","ram","disk","net"],"right":["cpu","battery"]}}"#);
        assert_eq!(s.widgets.left, ["cpu", "ram", "disk"]);
        assert_eq!(s.widgets.right, ["battery"]);
    }

    #[test]
    fn low_battery_snaps_to_5_50_in_steps_of_5() {
        let at = |v: u8| status_from(&format!(r#"{{"low_battery":{v}}}"#)).low_battery;
        assert_eq!((at(3), at(22), at(99)), (5, 20, 50));
    }

    #[test]
    fn bad_temp_unit_falls_back_to_celsius() {
        assert_eq!(status_from(r#"{"temp_unit":"kelvin"}"#).temp_unit, "celsius");
        assert_eq!(status_from(r#"{"temp_unit":"fahrenheit"}"#).temp_unit, "fahrenheit");
    }

    #[test]
    fn a_partial_island_is_filled_with_defaults() {
        let i: Island = serde_json::from_str(r#"{"screens":{"music":{"visualizer":false}}}"#).unwrap();
        assert!(!i.screens.music.visualizer);
        assert!(i.screens.music.enabled && i.screens.music.artwork);
        assert!(i.nav.wheel);
        assert_eq!(i.screens.notifications.banner_seconds, 5);
    }

    #[test]
    fn a_broken_order_is_repaired_to_every_screen_once() {
        let mut i = Island::default();
        i.nav.order = ["shelf", "bogus", "shelf", "MUSIC"].map(String::from).to_vec();
        i.normalize();
        assert_eq!(i.nav.order, ["shelf", "music", "status", "tools", "quick", "notifications"].map(String::from).to_vec());
        i.nav.order.clear();
        i.normalize();
        assert_eq!(i.nav.order, SCREEN_IDS.map(String::from).to_vec());
    }

    #[test]
    fn the_last_screen_cannot_all_be_off() {
        let mut i = Island::default();
        i.screens.status.enabled = false;
        i.screens.music.enabled = false;
        i.screens.tools.enabled = false;
        i.screens.quick.enabled = false;
        i.screens.notifications.enabled = false;
        i.screens.shelf.enabled = false;
        i.normalize();
        assert_eq!(i.enabled_flags(), [true, false, false, false, false, false]);
    }

    #[test]
    fn one_screen_on_is_left_alone() {
        let mut i = Island::default();
        i.screens.status.enabled = false;
        i.screens.music.enabled = false;
        i.screens.tools.enabled = false;
        i.screens.quick.enabled = false;
        i.screens.notifications.enabled = false;
        i.normalize();
        assert_eq!(i.enabled_flags(), [false, false, false, false, false, true]);
    }

    #[test]
    fn ranges_are_clamped_and_history_snaps_to_its_choices() {
        let mut i = Island::default();
        i.screens.notifications.banner_seconds = 1;
        i.screens.notifications.history_max = 30;
        i.screens.shelf.max_items = 0;
        i.normalize();
        assert_eq!((i.screens.notifications.banner_seconds, i.screens.notifications.history_max, i.screens.shelf.max_items), (3, 25, 1));
        i.screens.notifications.banner_seconds = 99;
        i.screens.notifications.history_max = 80;
        i.screens.shelf.max_items = 5000;
        i.normalize();
        assert_eq!((i.screens.notifications.banner_seconds, i.screens.notifications.history_max, i.screens.shelf.max_items), (10, 100, 100));
    }

    #[test]
    fn click_and_fullscreen_modes_use_lowercase_names() {
        let i: Island = serde_json::from_str(r#"{"fullscreen":"hidden","nav":{"click":"player"}}"#).unwrap();
        assert_eq!(i.fullscreen, FullscreenMode::Hidden);
        assert_eq!(i.nav.click, ClickMode::Player);
        let text = serde_json::to_string(&Island::default()).unwrap();
        assert!(text.contains(r#""fullscreen":"edge""#) && text.contains(r#""click":"smart""#));
    }
}
