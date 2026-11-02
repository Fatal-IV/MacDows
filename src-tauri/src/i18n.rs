/// Tray menu labels `(settings, quit)` for a language code; unknown codes fall back to English.
pub fn tray_labels(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "tr" => ("Ayarlar", "Çıkış"),
        _ => ("Settings", "Quit"),
    }
}

/// Tray menu label that brings the Windows taskbar back.
pub fn tray_restore_label(lang: &str) -> &'static str {
    match lang {
        "tr" => "Windows görev çubuğunu geri getir",
        _ => "Restore Windows taskbar",
    }
}

/// The one-time explanation shown before MacDows hides the taskbar for the first time: `(title, text)`.
pub fn taskbar_notice(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "tr" => (
            "MacDows",
            "MacDows, Windows görev çubuğunun yerine kendi dock'unu koyacak.\n\nBunu istediğiniz zaman geri alabilirsiniz: Ayarlar'da \"Windows görev çubuğunu değiştir\" anahtarını kapatın, tepsi menüsünden \"Windows görev çubuğunu geri getir\"i seçin ya da macdows.exe --restore-taskbar komutunu çalıştırın.\n\nWin tuşu Başlat menüsünü yine açar; dock'un sağ ucundaki düğmeler bildirimleri, hızlı ayarları ve gizli simgeleri açar.",
        ),
        _ => (
            "MacDows",
            "MacDows is about to replace the Windows taskbar with its own dock.\n\nYou can undo this at any time: turn off \"Replace the Windows taskbar\" in Settings, choose \"Restore Windows taskbar\" in the tray menu, or run macdows.exe --restore-taskbar.\n\nThe Win key still opens the Start menu; the buttons at the right end of the dock open notifications, quick settings and the hidden icons.",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turkish_labels() {
        assert_eq!(tray_labels("tr"), ("Ayarlar", "Çıkış"));
    }

    #[test]
    fn english_labels() {
        assert_eq!(tray_labels("en"), ("Settings", "Quit"));
    }

    #[test]
    fn unknown_language_falls_back_to_english() {
        assert_eq!(tray_labels("de"), ("Settings", "Quit"));
        assert_eq!(tray_labels(""), ("Settings", "Quit"));
    }
}

#[cfg(test)]
mod taskbar_text_tests {
    use super::*;

    #[test]
    fn the_tray_has_a_restore_taskbar_item_in_both_languages() {
        assert_eq!(tray_restore_label("tr"), "Windows görev çubuğunu geri getir");
        assert_eq!(tray_restore_label("en"), "Restore Windows taskbar");
        assert_eq!(tray_restore_label("de"), "Restore Windows taskbar");
    }

    #[test]
    fn the_first_run_notice_explains_how_to_undo_it() {
        let (_, tr) = taskbar_notice("tr");
        assert!(tr.contains("--restore-taskbar") && tr.contains("Ayarlar"));
        let (_, en) = taskbar_notice("en");
        assert!(en.contains("--restore-taskbar") && en.contains("Settings"));
        assert_eq!(taskbar_notice("de"), taskbar_notice("en"));
    }
}
