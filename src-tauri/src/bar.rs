//! What the label of a bar window means. The island and the dock of the primary monitor are `notch` and `dock`; those of another
//! monitor are `notch-2`, `dock-3`, ... (built at run time, see `monitors.rs`). Everything that needs "the bars" asks here.

use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BarKind {
    Notch,
    Dock,
}

/// The kind and the number of a bar label: `notch` is `(Notch, None)`, `dock-2` is `(Dock, Some(2))`. Anything else is not a bar.
pub fn parse(label: &str) -> Option<(BarKind, Option<u32>)> {
    let (kind, rest) = if let Some(r) = label.strip_prefix("notch") {
        (BarKind::Notch, r)
    } else if let Some(r) = label.strip_prefix("dock") {
        (BarKind::Dock, r)
    } else {
        return None;
    };
    if rest.is_empty() {
        return Some((kind, None));
    }
    let digits = rest.strip_prefix('-')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok().map(|n| (kind, Some(n)))
}

pub fn label(kind: BarKind, n: Option<u32>) -> String {
    let base = match kind {
        BarKind::Notch => "notch",
        BarKind::Dock => "dock",
    };
    match n {
        Some(n) => format!("{base}-{n}"),
        None => base.to_string(),
    }
}

pub fn kind_of(label: &str) -> Option<BarKind> {
    parse(label).map(|(kind, _)| kind)
}

pub fn is_bar(label: &str) -> bool {
    parse(label).is_some()
}

/// Labels of the bar windows that exist right now, sorted.
pub fn bar_labels(app: &AppHandle) -> Vec<String> {
    let mut labels: Vec<String> = app.webview_windows().keys().filter(|l| is_bar(l)).cloned().collect();
    labels.sort();
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_primary_pair_has_plain_labels() {
        assert_eq!(parse("notch"), Some((BarKind::Notch, None)));
        assert_eq!(parse("dock"), Some((BarKind::Dock, None)));
    }

    #[test]
    fn other_monitors_have_numbered_labels() {
        assert_eq!(parse("dock-2"), Some((BarKind::Dock, Some(2))));
        assert_eq!(parse("notch-13"), Some((BarKind::Notch, Some(13))));
    }

    #[test]
    fn anything_else_is_not_a_bar() {
        for l in ["notch-", "notch-x", "dock-2x", "docker", "notches", "settings", "preview", "tray", "", "dock--2"] {
            assert_eq!(parse(l), None, "{l:?}");
            assert!(!is_bar(l));
        }
    }

    #[test]
    fn labels_round_trip() {
        for (k, n) in [(BarKind::Notch, None), (BarKind::Dock, None), (BarKind::Dock, Some(3)), (BarKind::Notch, Some(10))] {
            assert_eq!(parse(&label(k, n)), Some((k, n)));
        }
        assert_eq!(label(BarKind::Dock, Some(3)), "dock-3");
        assert_eq!(label(BarKind::Notch, None), "notch");
        assert_eq!(kind_of("dock-4"), Some(BarKind::Dock));
        assert_eq!(kind_of("tray"), None);
    }
}
