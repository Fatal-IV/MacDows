//! Pure decisions of the installer. No Windows calls here, so everything is unit tested.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
}

/// Turkish for `tr`, `tr-TR`, `TR`; English for everything else, including an empty string.
pub fn pick_lang(locale: &str) -> Lang {
    let primary = locale.split(['-', '_']).next().unwrap_or("");
    if primary.eq_ignore_ascii_case("tr") {
        Lang::Tr
    } else {
        Lang::En
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Nothing installed yet.
    Fresh,
    /// Same version installed.
    Reinstall,
    /// An older version is installed (offer the upgrade).
    Older,
    /// A newer version is installed (never downgrade).
    Newer,
}

impl Decision {
    /// Stable names used by the UI.
    pub fn as_str(self) -> &'static str {
        match self {
            Decision::Fresh => "fresh",
            Decision::Reinstall => "reinstall",
            Decision::Older => "older",
            Decision::Newer => "newer",
        }
    }
}

fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let mut parts = v.trim().split('.');
    let mut next = || parts.next().and_then(|p| p.parse::<u32>().ok());
    Some((next()?, next()?, next()?))
}

/// An installed version that cannot be parsed counts as older, so a broken entry gets repaired.
pub fn decide(installed: Option<&str>, bundled: &str) -> Decision {
    let Some(installed) = installed else { return Decision::Fresh };
    let (Some(have), Some(want)) = (parse_version(installed), parse_version(bundled)) else {
        return Decision::Older;
    };
    match have.cmp(&want) {
        std::cmp::Ordering::Equal => Decision::Reinstall,
        std::cmp::Ordering::Less => Decision::Older,
        std::cmp::Ordering::Greater => Decision::Newer,
    }
}

/// One sentence for a non-zero exit code of the NSIS package. Zero is success and must not be passed.
pub fn exit_code_message(lang: Lang, code: i32) -> String {
    debug_assert!(code != 0, "exit code 0 means success");
    match (lang, code) {
        (Lang::Tr, 1) => "Kurulum iptal edildi.".to_string(),
        (Lang::En, 1) => "Setup was cancelled.".to_string(),
        (Lang::Tr, c) => format!("Kurulum {c} koduyla durdu."),
        (Lang::En, c) => format!("The installer stopped with code {c}."),
    }
}

/// `pv` is the WebView2 runtime version from the registry; missing, empty or all zeros mean "not installed".
pub fn webview2_present(pv: Option<&str>) -> bool {
    matches!(pv, Some(v) if !v.trim().is_empty() && v.trim() != "0.0.0.0")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Preparing,
    Installing,
    Finishing,
    Done,
}

/// Staged on purpose: silent NSIS reports no real progress, so no measured percentage is ever shown.
pub fn stage_percent(stage: Stage) -> u8 {
    match stage {
        Stage::Preparing => 15,
        Stage::Installing => 60,
        Stage::Finishing => 95,
        Stage::Done => 100,
    }
}

pub fn log_line(timestamp: &str, level: &str, message: &str) -> String {
    format!("[{timestamp}] {} {message}", level.to_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_is_turkish_only_for_tr_locales() {
        assert_eq!(pick_lang("tr-TR"), Lang::Tr);
        assert_eq!(pick_lang("TR"), Lang::Tr);
        assert_eq!(pick_lang("de-DE"), Lang::En);
        assert_eq!(pick_lang("en-US"), Lang::En);
        assert_eq!(pick_lang(""), Lang::En);
    }

    #[test]
    fn decision_compares_installed_with_bundled() {
        assert_eq!(decide(None, "0.1.0"), Decision::Fresh);
        assert_eq!(decide(Some("0.1.0"), "0.1.0"), Decision::Reinstall);
        assert_eq!(decide(Some("0.0.9"), "0.1.0"), Decision::Older);
        assert_eq!(decide(Some("0.2.0"), "0.1.0"), Decision::Newer);
        assert_eq!(decide(Some("0.10.0"), "0.9.0"), Decision::Newer);
        assert_eq!(decide(Some("garbage"), "0.1.0"), Decision::Older);
    }

    #[test]
    fn exit_code_messages_are_localized_and_mention_unknown_codes() {
        assert!(exit_code_message(Lang::En, 2).contains('2'));
        assert!(exit_code_message(Lang::Tr, 1).to_lowercase().contains("kurulum"));
        assert!(exit_code_message(Lang::Tr, 77).contains("77"));
        assert!(exit_code_message(Lang::En, 77).contains("77"));
    }

    #[test]
    #[should_panic]
    fn exit_code_zero_is_a_caller_bug() {
        let _ = exit_code_message(Lang::En, 0);
    }

    #[test]
    fn webview2_value_interpretation() {
        assert!(!webview2_present(None));
        assert!(!webview2_present(Some("")));
        assert!(!webview2_present(Some("0.0.0.0")));
        assert!(webview2_present(Some("131.0.2903.51")));
    }

    #[test]
    fn stages_map_to_the_agreed_percentages() {
        assert_eq!(stage_percent(Stage::Preparing), 15);
        assert_eq!(stage_percent(Stage::Installing), 60);
        assert_eq!(stage_percent(Stage::Finishing), 95);
        assert_eq!(stage_percent(Stage::Done), 100);
    }

    #[test]
    fn log_lines_are_timestamped_and_level_uppercased() {
        assert_eq!(log_line("12:00:00", "error", "boom"), "[12:00:00] ERROR boom");
    }
}

#[cfg(test)]
mod decision_name_tests {
    use super::*;

    #[test]
    fn decision_names_are_stable_for_the_ui() {
        assert_eq!(Decision::Fresh.as_str(), "fresh");
        assert_eq!(Decision::Reinstall.as_str(), "reinstall");
        assert_eq!(Decision::Older.as_str(), "older");
        assert_eq!(Decision::Newer.as_str(), "newer");
    }
}
