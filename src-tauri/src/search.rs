//! Backend of the MacDows search menu: apps, files and folders, Windows settings pages, a calculator and web search.
//!
//! Files come from a small index of the user's own folders (Desktop, Documents, Downloads, Pictures, Videos, Music),
//! built in the background and refreshed every ten minutes. Opening anything is limited to what the search itself
//! returned: a known settings page, a file that is in the index, or a web search URL built here.

use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

use crate::launcher::{self, AppEntry};

const FILE_RESULTS: usize = 6;
const APP_RESULTS: usize = 6;
const SETTING_RESULTS: usize = 4;
const INDEX_MAX_ENTRIES: usize = 250_000;
const INDEX_MAX_DEPTH: usize = 6;
const INDEX_MAX_AGE: Duration = Duration::from_secs(600);
const SKIP_DIRS: &[&str] = &["node_modules", "target", "$recycle.bin", "appdata", "__pycache__", "dist", "build"];

// ---------------------------------------------------------------------------------------------
// Matching
// ---------------------------------------------------------------------------------------------

/// Lower-cases and removes Turkish and other common diacritics, so "ayarlar", "AYARLAR" and "çalışma" / "calisma"
/// all compare the way people type them.
pub fn fold(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'ç' => 'c',
            'ğ' => 'g',
            'ı' | 'i' | 'î' | 'í' | 'ì' => 'i',
            'ö' | 'ó' | 'ò' | 'ô' => 'o',
            'ş' => 's',
            'ü' | 'ú' | 'ù' | 'û' => 'u',
            'á' | 'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            other => other,
        })
        .filter(|c| *c != '\u{307}')
        .collect()
}

/// 0 = the name starts with the query, 1 = a word in it does, 2 = it contains the query. Lower is better.
pub fn match_rank(folded_name: &str, folded_query: &str) -> Option<u8> {
    if folded_query.is_empty() {
        return None;
    }
    if folded_name.starts_with(folded_query) {
        return Some(0);
    }
    if folded_name.split(|c: char| !c.is_alphanumeric()).any(|w| w.starts_with(folded_query)) {
        return Some(1);
    }
    folded_name.contains(folded_query).then_some(2)
}

// ---------------------------------------------------------------------------------------------
// Calculator
// ---------------------------------------------------------------------------------------------

struct Parser<'a> {
    chars: Vec<char>,
    pos: usize,
    _src: &'a str,
}

impl Parser<'_> {
    fn peek(&mut self) -> Option<char> {
        while self.chars.get(self.pos).is_some_and(|c| c.is_whitespace()) {
            self.pos += 1;
        }
        self.chars.get(self.pos).copied()
    }

    fn expr(&mut self) -> Option<f64> {
        let mut value = self.term()?;
        while let Some(op @ ('+' | '-')) = self.peek() {
            self.pos += 1;
            let rhs = self.term()?;
            value = if op == '+' { value + rhs } else { value - rhs };
        }
        Some(value)
    }

    fn term(&mut self) -> Option<f64> {
        let mut value = self.power()?;
        while let Some(op @ ('*' | '/' | 'x' | 'X' | '×' | '÷')) = self.peek() {
            self.pos += 1;
            let rhs = self.power()?;
            value = if matches!(op, '/' | '÷') { value / rhs } else { value * rhs };
        }
        Some(value)
    }

    fn power(&mut self) -> Option<f64> {
        let base = self.unary()?;
        if self.peek() == Some('^') {
            self.pos += 1;
            let exp = self.power()?;
            return Some(base.powf(exp));
        }
        Some(base)
    }

    fn unary(&mut self) -> Option<f64> {
        match self.peek()? {
            '-' => {
                self.pos += 1;
                Some(-self.unary()?)
            }
            '+' => {
                self.pos += 1;
                self.unary()
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Option<f64> {
        if self.peek()? == '(' {
            self.pos += 1;
            let value = self.expr()?;
            if self.peek()? != ')' {
                return None;
            }
            self.pos += 1;
            return Some(value);
        }
        let start = self.pos;
        while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit() || *c == '.' || *c == ',') {
            self.pos += 1;
        }
        let text: String = self.chars[start..self.pos].iter().map(|c| if *c == ',' { '.' } else { *c }).collect();
        text.parse().ok()
    }
}

/// Evaluates a typed arithmetic expression. `None` unless it is a complete expression with at least one operator,
/// so plain words and single numbers never show a calculator result.
pub fn calculate(input: &str) -> Option<String> {
    let trimmed = input.trim().trim_start_matches('=').trim();
    let has_operator = trimmed.char_indices().any(|(i, c)| matches!(c, '+' | '*' | '/' | '^' | '×' | '÷') || (c == '-' && i > 0) || (matches!(c, 'x' | 'X') && i > 0));
    if trimmed.is_empty() || !has_operator || !trimmed.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut parser = Parser { chars: trimmed.chars().collect(), pos: 0, _src: trimmed };
    let value = parser.expr()?;
    if parser.peek().is_some() || !value.is_finite() {
        return None;
    }
    Some(format_number(value))
}

fn format_number(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        return format!("{}", v as i64);
    }
    let s = format!("{:.10}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ---------------------------------------------------------------------------------------------
// Windows settings pages
// ---------------------------------------------------------------------------------------------

/// (ms-settings page, Turkish name, English name)
const SETTINGS: &[(&str, &str, &str)] = &[
    ("display", "Ekran", "Display"),
    ("sound", "Ses", "Sound"),
    ("notifications", "Bildirimler", "Notifications"),
    ("powersleep", "Güç ve uyku", "Power & sleep"),
    ("batterysaver", "Pil tasarrufu", "Battery saver"),
    ("storagesense", "Depolama", "Storage"),
    ("nightlight", "Gece ışığı", "Night light"),
    ("bluetooth", "Bluetooth ve cihazlar", "Bluetooth & devices"),
    ("printers", "Yazıcılar ve tarayıcılar", "Printers & scanners"),
    ("mousetouchpad", "Fare", "Mouse"),
    ("network-status", "Ağ ve internet", "Network & internet"),
    ("network-wifi", "Wi-Fi", "Wi-Fi"),
    ("network-vpn", "VPN", "VPN"),
    ("personalization", "Kişiselleştirme", "Personalization"),
    ("personalization-background", "Arka plan", "Background"),
    ("colors", "Renkler", "Colors"),
    ("themes", "Temalar", "Themes"),
    ("lockscreen", "Kilit ekranı", "Lock screen"),
    ("taskbar", "Görev çubuğu", "Taskbar"),
    ("personalization-start", "Başlat", "Start"),
    ("appsfeatures", "Yüklü uygulamalar", "Installed apps"),
    ("defaultapps", "Varsayılan uygulamalar", "Default apps"),
    ("startupapps", "Başlangıç uygulamaları", "Startup apps"),
    ("yourinfo", "Hesap bilgileriniz", "Your info"),
    ("signinoptions", "Oturum açma seçenekleri", "Sign-in options"),
    ("dateandtime", "Tarih ve saat", "Date & time"),
    ("regionlanguage", "Dil ve bölge", "Language & region"),
    ("easeofaccess", "Erişilebilirlik", "Accessibility"),
    ("privacy", "Gizlilik ve güvenlik", "Privacy & security"),
    ("windowsupdate", "Windows Update", "Windows Update"),
    ("recovery", "Kurtarma", "Recovery"),
    ("about", "Hakkında", "About"),
    ("multitasking", "Çoklu görev", "Multitasking"),
    ("clipboard", "Pano", "Clipboard"),
    ("gaming", "Oyun", "Gaming"),
    ("privacy-webcam", "Kamera gizliliği", "Camera privacy"),
    ("privacy-microphone", "Mikrofon gizliliği", "Microphone privacy"),
];

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct SettingHit {
    pub page: String,
    pub name: String,
}

fn search_settings(query: &str) -> Vec<SettingHit> {
    let q = fold(query);
    let mut hits: Vec<(u8, SettingHit)> = SETTINGS
        .iter()
        .filter_map(|(page, tr, en)| {
            let rank = match_rank(&fold(tr), &q).into_iter().chain(match_rank(&fold(en), &q)).min()?;
            Some((rank, SettingHit { page: page.to_string(), name: tr.to_string() }))
        })
        .collect();
    hits.sort_by_key(|(rank, _)| *rank);
    hits.into_iter().take(SETTING_RESULTS).map(|(_, h)| h).collect()
}

// ---------------------------------------------------------------------------------------------
// File index
// ---------------------------------------------------------------------------------------------

#[derive(Clone)]
struct FileEntry {
    name: String,
    folded: String,
    path: String,
    is_dir: bool,
    modified: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileHit {
    pub path: String,
    pub name: String,
    pub folder: String,
    pub is_dir: bool,
}

static INDEX: Mutex<Option<(Instant, Vec<FileEntry>)>> = Mutex::new(None);

fn indexed_roots() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) else { return Vec::new() };
    ["Desktop", "Documents", "Downloads", "Pictures", "Videos", "Music"].iter().map(|d| home.join(d)).filter(|p| p.is_dir()).collect()
}

fn build_index() -> Vec<FileEntry> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = indexed_roots().into_iter().map(|p| (p, 0)).collect();
    while let Some((dir, depth)) = stack.pop() {
        let Ok(read) = std::fs::read_dir(&dir) else { continue };
        for entry in read.flatten() {
            if out.len() >= INDEX_MAX_ENTRIES {
                return out;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name.starts_with('~') || name.eq_ignore_ascii_case("desktop.ini") {
                continue;
            }
            let Ok(meta) = entry.metadata() else { continue };
            let is_dir = meta.is_dir();
            if is_dir && (depth + 1 > INDEX_MAX_DEPTH || SKIP_DIRS.contains(&name.to_lowercase().as_str())) {
                continue;
            }
            let modified = meta.modified().ok().and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_secs());
            let path = entry.path();
            if is_dir {
                stack.push((path.clone(), depth + 1));
            }
            out.push(FileEntry { folded: fold(&name), name, path: path.to_string_lossy().to_string(), is_dir, modified });
        }
    }
    out
}

fn refresh_index() {
    let entries = build_index();
    *INDEX.lock().unwrap() = Some((Instant::now(), entries));
}

/// Builds the file index in the background at startup and keeps it fresh.
pub fn warm_up() {
    std::thread::spawn(|| loop {
        refresh_index();
        std::thread::sleep(INDEX_MAX_AGE);
    });
}

fn search_files(query: &str) -> Vec<FileHit> {
    let q = fold(query);
    let guard = INDEX.lock().unwrap();
    let Some((_, entries)) = guard.as_ref() else { return Vec::new() };
    let mut hits: Vec<(u8, u64, &FileEntry)> = entries.iter().filter_map(|e| Some((match_rank(&e.folded, &q)?, e.modified, e))).collect();
    // Best match first; among equals, the most recently changed.
    hits.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    hits.into_iter()
        .take(FILE_RESULTS)
        .map(|(_, _, e)| FileHit {
            path: e.path.clone(),
            name: e.name.clone(),
            folder: Path::new(&e.path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default(),
            is_dir: e.is_dir,
        })
        .collect()
}

fn in_index(path: &str) -> bool {
    INDEX.lock().unwrap().as_ref().is_some_and(|(_, entries)| entries.iter().any(|e| e.path == path))
}

// ---------------------------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------------------------

#[derive(Serialize, Debug)]
pub struct SearchResults {
    pub calc: Option<String>,
    pub apps: Vec<AppEntry>,
    pub settings: Vec<SettingHit>,
    pub files: Vec<FileHit>,
}

#[tauri::command]
pub async fn search_all(query: String) -> SearchResults {
    tauri::async_runtime::spawn_blocking(move || {
        let query = query.trim().to_string();
        if query.is_empty() {
            return SearchResults { calc: None, apps: Vec::new(), settings: Vec::new(), files: Vec::new() };
        }
        SearchResults {
            calc: calculate(&query),
            apps: launcher::search_apps(&query, APP_RESULTS),
            settings: search_settings(&query),
            files: search_files(&query),
        }
    })
    .await
    .unwrap_or(SearchResults { calc: None, apps: Vec::new(), settings: Vec::new(), files: Vec::new() })
}

fn shell_open(target: &str) -> bool {
    let wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let r = unsafe { ShellExecuteW(None, windows::core::w!("open"), PCWSTR(wide.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    r.0 as isize > 32
}

#[tauri::command]
pub fn open_file(path: String) -> bool {
    in_index(&path) && shell_open(&path)
}

#[tauri::command]
pub fn open_setting(page: String) -> bool {
    SETTINGS.iter().any(|(p, _, _)| *p == page) && shell_open(&format!("ms-settings:{page}"))
}

/// Percent-encodes everything but unreserved characters.
pub fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[tauri::command]
pub fn open_web_search(query: String) -> bool {
    let q = query.trim();
    !q.is_empty() && shell_open(&format!("https://www.bing.com/search?q={}", url_encode(q)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_ignores_case_and_turkish_letters() {
        assert_eq!(fold("ÇALIŞMA Ğ İ ı Ö Ü"), "calisma g i i o u");
        assert_eq!(fold("Görev Çubuğu"), "gorev cubugu");
        assert_eq!(fold("İndirilenler"), "indirilenler");
    }

    #[test]
    fn names_rank_by_how_well_they_match() {
        assert_eq!(match_rank("ayarlar", "ay"), Some(0));
        assert_eq!(match_rank("windows update", "up"), Some(1));
        assert_eq!(match_rank("kisisellestirme", "sel"), Some(2));
        assert_eq!(match_rank("ayarlar", "xyz"), None);
        assert_eq!(match_rank("ayarlar", ""), None);
    }

    #[test]
    fn the_calculator_handles_precedence_parentheses_and_decimals() {
        assert_eq!(calculate("2+3*4").as_deref(), Some("14"));
        assert_eq!(calculate("(2+3)*4").as_deref(), Some("20"));
        assert_eq!(calculate("10/4").as_deref(), Some("2.5"));
        assert_eq!(calculate("2^3^2").as_deref(), Some("512"));
        assert_eq!(calculate("-5 + 2").as_deref(), Some("-3"));
        assert_eq!(calculate("1,5*2").as_deref(), Some("3"));
        assert_eq!(calculate("= 7 x 6").as_deref(), Some("42"));
        assert_eq!(calculate("0.1+0.2").as_deref(), Some("0.3"));
    }

    #[test]
    fn the_calculator_ignores_words_single_numbers_and_broken_input() {
        assert_eq!(calculate("42"), None);
        assert_eq!(calculate("chrome"), None);
        assert_eq!(calculate("1/0"), None);
        assert_eq!(calculate("2+"), None);
        assert_eq!(calculate("(2+3"), None);
        assert_eq!(calculate("windows 11"), None);
    }

    #[test]
    fn settings_are_found_by_turkish_or_english_names() {
        let hits = search_settings("gorev");
        assert_eq!(hits.first().map(|h| h.page.as_str()), Some("taskbar"));
        assert!(search_settings("bluetooth").iter().any(|h| h.page == "bluetooth"));
        assert!(search_settings("display").iter().any(|h| h.page == "display"));
        assert!(search_settings("zzzzqq").is_empty());
    }

    #[test]
    fn only_catalogued_settings_pages_can_be_opened() {
        assert!(!open_setting("../../evil".into()));
        assert!(!open_setting("calc".into()));
    }

    #[test]
    fn web_queries_are_percent_encoded() {
        assert_eq!(url_encode("hava durumu"), "hava%20durumu");
        assert_eq!(url_encode("a&b=c"), "a%26b%3Dc");
        assert_eq!(url_encode("çay"), "%C3%A7ay");
        assert_eq!(url_encode("Rust-lang_1.0~"), "Rust-lang_1.0~");
    }
}
