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

/// Why a custom icon could not be set, for the message box. `reason` is the English reason from `custom_icons::save`.
pub fn icon_error(lang: &str, reason: &str) -> String {
    let what = match (lang == "tr", reason) {
        (true, "not an image") => "Seçilen dosya bir resim değil.".to_string(),
        (true, "file is too big") => "Dosya çok büyük (en fazla 10 MB).".to_string(),
        (true, "file not found") => "Dosya bulunamadı.".to_string(),
        (true, "this program has no icon") => "Bu programın simgesi yok.".to_string(),
        (true, other) => other.to_string(),
        (false, "not an image") => "The chosen file is not an image.".to_string(),
        (false, "file is too big") => "The file is too big (10 MB at most).".to_string(),
        (false, "file not found") => "The file was not found.".to_string(),
        (false, "this program has no icon") => "This program has no icon.".to_string(),
        (false, other) => other.to_string(),
    };
    if lang == "tr" { format!("Simge değiştirilemedi. {what}") } else { format!("The icon could not be changed. {what}") }
}

/// Label of a dock right-click menu item, by the action's id; unknown ids give an empty label.
pub fn dock_menu_label(lang: &str, action: &str) -> &'static str {
    let tr = lang == "tr";
    match (action, tr) {
        ("new_window", true) => "Yeni pencere",
        ("new_window", false) => "New window",
        ("unpin", true) => "Dock'ten kaldır",
        ("unpin", false) => "Remove from Dock",
        ("pin", true) => "Dock'e sabitle",
        ("pin", false) => "Keep in Dock",
        ("change_icon", true) => "Simgeyi değiştir",
        ("change_icon", false) => "Change icon",
        ("reset_icon", true) => "Simgeyi sıfırla",
        ("reset_icon", false) => "Reset icon",
        ("add_app", true) => "Uygulama ekle",
        ("add_app", false) => "Add app",
        ("macdows_settings", true) => "MacDows Ayarları",
        ("macdows_settings", false) => "MacDows Settings",
        ("task_manager", true) => "Görev Yöneticisi",
        ("task_manager", false) => "Task Manager",
        _ => "",
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
