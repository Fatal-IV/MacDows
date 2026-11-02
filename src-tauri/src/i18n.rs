/// Tray menu labels `(settings, quit)` for a language code; unknown codes fall back to English.
pub fn tray_labels(lang: &str) -> (&'static str, &'static str) {
    match lang {
        "tr" => ("Ayarlar", "Çıkış"),
        _ => ("Settings", "Quit"),
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
