//! The charger was plugged in or pulled out, the battery is full, or it is running low: the island shows it the way it
//! shows a Wi-Fi or Bluetooth change (`link-changed` with `kind` "power"; `state` is "plugged", "unplugged", "full" or
//! "low"; `name` is the battery percent). What is true when MacDows starts is not announced.

use crate::links::Link;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub percent: u8,
    pub on_ac: bool,
    /// Windows says the battery is taking charge right now. On a charger it is off at 100 and also when a charge
    /// limit (battery protection) holds the battery at, say, 80.
    pub charging: bool,
}

/// Readings in a row (3 seconds apart) with the charger in, no charging and the percent unchanged before the battery
/// counts as held at its charge limit: about 15 seconds.
const LIMIT_READINGS: u32 = 5;
/// A charge limit is never set lower than this; below it "not charging" is a pause (a warm battery), not a limit.
const LIMIT_FLOOR: u8 = 60;

fn held_at_limit_shape(r: Reading) -> bool {
    r.on_ac && !r.charging && r.percent >= LIMIT_FLOOR
}

/// Compares each reading with the one before and says what changed. "full" and "low" are told once and again only
/// after the battery has left that state (it dropped below full, or went back above the limit, or the charger changed).
#[derive(Default)]
pub struct Tracker {
    last: Option<Reading>,
    full_told: bool,
    low_told: bool,
    /// Readings in a row with the charger in, not charging and the same percent.
    still: u32,
}

impl Tracker {
    pub fn feed(&mut self, now: Reading, low: u8) -> Vec<&'static str> {
        let mut events = Vec::new();
        match self.last {
            None => {
                self.full_told = now.on_ac && (now.percent >= 100 || held_at_limit_shape(now));
                self.low_told = !now.on_ac && now.percent <= low;
            }
            Some(before) => {
                if now.on_ac != before.on_ac {
                    if now.on_ac {
                        events.push("plugged");
                        self.low_told = false;
                    } else {
                        events.push("unplugged");
                        self.full_told = false;
                    }
                }
                self.still = if held_at_limit_shape(now) && now.percent == before.percent { self.still + 1 } else { 0 };
                if now.on_ac && (now.percent >= 100 || self.still >= LIMIT_READINGS) {
                    if !self.full_told {
                        events.push("full");
                        self.full_told = true;
                    }
                } else if !now.on_ac || (now.charging && now.percent < 100) {
                    // Charging again (or the charger is out): it can be told again. A percent that only drifts by one
                    // while a limit holds the battery does not count.
                    self.full_told = false;
                }
                if !now.on_ac && now.percent <= low {
                    if !self.low_told {
                        events.push("low");
                        self.low_told = true;
                    }
                } else {
                    self.low_told = false;
                }
            }
        }
        self.last = Some(now);
        events
    }
}

fn read() -> Option<Reading> {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut power = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut power) }.ok()?;
    // 128: this computer has no battery (a desktop PC): nothing to announce.
    if power.BatteryFlag & 128 != 0 {
        return None;
    }
    Some(Reading { percent: crate::taskbar::battery_percent_from(power.BatteryLifePercent)?, on_ac: power.ACLineStatus == 1, charging: power.BatteryFlag & 8 != 0 })
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut tracker = Tracker::default();
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let Some(now) = read() else { continue };
            let low = app.state::<crate::AppSettings>().0.lock().unwrap().island.screens.status.low_battery;
            let events = tracker.feed(now, low);
            if !events.is_empty() {
                // Pulling the charger can blank the screen for a moment (the display changes its power mode): the
                // notice waits until the picture is back, or it would be shown while nobody can see it.
                std::thread::sleep(Duration::from_millis(1500));
            }
            for state in events {
                let _ = app.emit("link-changed", Link { kind: "power", name: now.percent.to_string(), state });
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reading as a normal battery gives it: charging whenever it is on the charger and below 100.
    const R: fn(u8, bool) -> Reading = |percent, on_ac| Reading { percent, on_ac, charging: on_ac && percent < 100 };
    /// On the charger, not charging: what a charge limit looks like.
    const HELD: fn(u8) -> Reading = |percent| Reading { percent, on_ac: true, charging: false };

    #[test]
    fn the_first_reading_only_sets_the_baseline() {
        let mut t = Tracker::default();
        assert!(t.feed(R(10, false), 20).is_empty());
        assert!(t.feed(R(10, false), 20).is_empty(), "already low at start: not told");
    }

    #[test]
    fn plugging_in_and_pulling_out_are_told() {
        let mut t = Tracker::default();
        t.feed(R(60, false), 20);
        assert_eq!(t.feed(R(60, true), 20), vec!["plugged"]);
        assert!(t.feed(R(61, true), 20).is_empty());
        assert_eq!(t.feed(R(61, false), 20), vec!["unplugged"]);
    }

    #[test]
    fn full_is_told_once_when_the_charge_reaches_100() {
        let mut t = Tracker::default();
        t.feed(R(98, true), 20);
        assert!(t.feed(R(99, true), 20).is_empty());
        assert_eq!(t.feed(R(100, true), 20), vec!["full"]);
        assert!(t.feed(R(100, true), 20).is_empty());
        // After a drop below 100 (charger pulled, battery used) it can be told again.
        assert_eq!(t.feed(R(100, false), 20), vec!["unplugged"]);
        t.feed(R(99, false), 20);
        t.feed(R(99, true), 20);
        assert_eq!(t.feed(R(100, true), 20), vec!["full"]);
    }

    #[test]
    fn low_is_told_once_when_the_battery_falls_to_the_limit() {
        let mut t = Tracker::default();
        t.feed(R(22, false), 20);
        assert!(t.feed(R(21, false), 20).is_empty());
        assert_eq!(t.feed(R(20, false), 20), vec!["low"]);
        assert!(t.feed(R(19, false), 20).is_empty());
        // Charging raises it above the limit; falling to it again is told again.
        assert_eq!(t.feed(R(19, true), 20), vec!["plugged"]);
        t.feed(R(40, true), 20);
        t.feed(R(40, false), 20);
        assert_eq!(t.feed(R(20, false), 20), vec!["low"]);
    }

    #[test]
    fn a_charger_that_is_not_plugged_in_is_never_called_low_while_charging() {
        let mut t = Tracker::default();
        t.feed(R(15, true), 20);
        assert!(t.feed(R(16, true), 20).is_empty());
    }

    #[test]
    fn pulling_the_charger_at_a_low_level_tells_both() {
        let mut t = Tracker::default();
        t.feed(R(15, true), 20);
        assert_eq!(t.feed(R(15, false), 20), vec!["unplugged", "low"]);
    }

    #[test]
    fn a_battery_held_at_its_charge_limit_counts_as_full() {
        let mut t = Tracker::default();
        t.feed(R(78, true), 20);
        t.feed(R(79, true), 20);
        assert!(t.feed(HELD(80), 20).is_empty());
        for _ in 0..4 {
            assert!(t.feed(HELD(80), 20).is_empty(), "not yet: it may only be a pause");
        }
        assert_eq!(t.feed(HELD(80), 20), vec!["full"]);
        assert!(t.feed(HELD(80), 20).is_empty(), "told once");
        // A drift by one percent while the limit holds is not a new charge.
        assert!(t.feed(HELD(79), 20).is_empty());
        for _ in 0..6 {
            assert!(t.feed(HELD(79), 20).is_empty());
        }
    }

    #[test]
    fn a_charge_limit_notice_comes_again_after_charging_resumes() {
        let mut t = Tracker::default();
        t.feed(R(78, true), 20);
        for _ in 0..6 {
            t.feed(HELD(80), 20);
        }
        t.feed(R(70, true), 20);
        let mut told = 0;
        for _ in 0..8 {
            told += t.feed(HELD(80), 20).iter().filter(|e| **e == "full").count();
        }
        assert_eq!(told, 1);
    }

    #[test]
    fn a_pause_below_the_floor_or_a_falling_percent_is_not_a_limit() {
        let mut t = Tracker::default();
        t.feed(R(40, true), 20);
        for _ in 0..8 {
            assert!(t.feed(HELD(40), 20).is_empty(), "40% held: a warm battery, not a limit");
        }
        let mut t = Tracker::default();
        t.feed(R(90, true), 20);
        for p in (80..90).rev() {
            assert!(t.feed(HELD(p), 20).is_empty(), "the percent keeps falling: the charger cannot keep up");
        }
    }

    #[test]
    fn already_held_at_a_limit_when_started_is_not_told() {
        let mut t = Tracker::default();
        t.feed(HELD(80), 20);
        for _ in 0..8 {
            assert!(t.feed(HELD(80), 20).is_empty());
        }
    }
}
