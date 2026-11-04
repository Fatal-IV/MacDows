//! Custom icons for dock apps: the user picks an image (or a program) and MacDows keeps its own 128x128 copy.
use base64::Engine;
use image::{imageops, DynamicImage, ImageFormat, RgbaImage};
use std::path::{Path, PathBuf};

const SIDE: u32 = 128;
const MAX_BYTES: u64 = 10 * 1024 * 1024;

pub fn icons_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows").join("icons")
}

/// The same key always gives the same name, whatever its case (FNV-1a, so it does not change between builds).
pub fn icon_file_name(key: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in key.to_lowercase().bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}.png")
}

/// Scales the image to fit a `side` x `side` square, keeps its proportions, and pads the rest transparent.
pub fn fit_to_square(img: DynamicImage, side: u32) -> RgbaImage {
    let (w, h) = (img.width().max(1), img.height().max(1));
    let scale = side as f32 / w.max(h) as f32;
    let (nw, nh) = (((w as f32 * scale).round() as u32).clamp(1, side), ((h as f32 * scale).round() as u32).clamp(1, side));
    let resized = imageops::resize(&img.to_rgba8(), nw, nh, imageops::FilterType::Lanczos3);
    let mut canvas = RgbaImage::new(side, side);
    imageops::overlay(&mut canvas, &resized, ((side - nw) / 2) as i64, ((side - nh) / 2) as i64);
    canvas
}

/// Reads the source as an image; a program (`.exe`) gives its own icon.
fn read_source(source: &Path) -> Result<DynamicImage, String> {
    let meta = std::fs::metadata(source).map_err(|_| "file not found".to_string())?;
    if meta.len() > MAX_BYTES {
        return Err("file is too big".into());
    }
    let is_exe = source.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe"));
    if is_exe {
        let b64 = crate::windows_list::icon_png_base64(&source.to_string_lossy()).ok_or("this program has no icon")?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).map_err(|e| e.to_string())?;
        return image::load_from_memory(&bytes).map_err(|e| e.to_string());
    }
    image::open(source).map_err(|_| "not an image".to_string())
}

/// Saves the 128x128 copy of `source` in `dir` under the key's file name and returns that name.
pub fn save_to(dir: &Path, key: &str, source: &Path) -> Result<String, String> {
    let img = read_source(source)?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let name = icon_file_name(key);
    fit_to_square(img, SIDE).save_with_format(dir.join(&name), ImageFormat::Png).map_err(|e| e.to_string())?;
    Ok(name)
}

pub fn save(key: &str, source: &Path) -> Result<String, String> {
    save_to(&icons_dir(), key, source)
}

pub fn remove(file: &str) {
    // Only a plain file name inside the icons folder is ever deleted.
    if Path::new(file).file_name().and_then(|n| n.to_str()) == Some(file) {
        let _ = std::fs::remove_file(icons_dir().join(file));
    }
}

fn load_base64(file: &str) -> Option<String> {
    if Path::new(file).file_name().and_then(|n| n.to_str()) != Some(file) {
        return None;
    }
    let bytes = std::fs::read(icons_dir().join(file)).ok()?;
    Some(base64::engine::general_purpose::STANDARD.encode(bytes))
}

/// Key (lowercase) to the PNG of its custom icon; icons whose file is missing are left out.
#[tauri::command]
pub fn get_custom_icons(app: tauri::AppHandle) -> std::collections::HashMap<String, String> {
    use tauri::Manager;
    let saved = app.state::<crate::AppSettings>().0.lock().unwrap().dock.custom_icons.clone();
    saved.into_iter().filter_map(|(key, file)| load_base64(&file).map(|png| (key.to_lowercase(), png))).collect()
}

/// The standard "open file" dialog for choosing an icon. `None` when the user cancels.
pub fn choose_file() -> Option<PathBuf> {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_PATHMUSTEXIST, OPENFILENAMEW};
    // "Images\0*.png;*.jpg;*.jpeg;*.ico;*.bmp;*.exe\0\0"
    let filter: Vec<u16> = "Images and programs\0*.png;*.jpg;*.jpeg;*.ico;*.bmp;*.exe\0\0".encode_utf16().collect();
    let mut buf = [0u16; 1040];
    let mut ofn = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrFile: windows::core::PWSTR(buf.as_mut_ptr()),
        nMaxFile: buf.len() as u32,
        Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
        ..Default::default()
    };
    if !unsafe { GetOpenFileNameW(&mut ofn) }.as_bool() {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("macdows-icons-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_file_name_is_stable_and_ignores_case() {
        assert_eq!(icon_file_name("C:/Pins/Edge.lnk"), icon_file_name("c:/pins/edge.LNK"));
        assert_ne!(icon_file_name("a"), icon_file_name("b"));
        assert!(icon_file_name("a").ends_with(".png"));
        // Pinned on a value, so a build that changed the hash would orphan every saved icon.
        assert_eq!(icon_file_name("a"), "af63dc4c8601ec8c.png");
    }

    #[test]
    fn a_wide_image_is_fitted_with_transparent_bars_above_and_below() {
        let wide = DynamicImage::ImageRgba8(RgbaImage::from_pixel(200, 100, image::Rgba([255, 0, 0, 255])));
        let out = fit_to_square(wide, 128);
        assert_eq!((out.width(), out.height()), (128, 128));
        assert_eq!(out.get_pixel(64, 2)[3], 0, "top bar is transparent");
        assert_eq!(out.get_pixel(64, 64)[3], 255, "the middle is the picture");
        assert_eq!(out.get_pixel(64, 125)[3], 0, "bottom bar is transparent");
    }

    #[test]
    fn a_file_that_is_not_an_image_is_rejected() {
        let dir = temp_dir("text");
        let file = dir.join("note.png");
        std::fs::write(&file, b"this is not an image").unwrap();
        assert_eq!(save_to(&dir, "k", &file), Err("not an image".to_string()));
        assert!(!dir.join(icon_file_name("k")).exists());
    }

    #[test]
    fn a_missing_file_is_rejected() {
        let dir = temp_dir("missing");
        assert!(save_to(&dir, "k", &dir.join("nope.png")).is_err());
    }

    #[test]
    fn a_picture_is_saved_as_a_128_png() {
        let dir = temp_dir("ok");
        let src = dir.join("in.png");
        RgbaImage::from_pixel(300, 300, image::Rgba([0, 128, 255, 255])).save(&src).unwrap();
        let name = save_to(&dir.join("icons"), "Key", &src).unwrap();
        let saved = image::open(dir.join("icons").join(&name)).unwrap();
        assert_eq!((saved.width(), saved.height()), (128, 128));
    }

    #[test]
    fn only_plain_file_names_are_loaded_or_removed() {
        assert!(load_base64("../settings.json").is_none());
        remove("../settings.json"); // must not delete anything outside the icons folder
    }
}
