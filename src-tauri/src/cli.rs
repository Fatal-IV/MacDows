//! Command line flags that must work without opening any window.

#[derive(Debug, PartialEq, Eq)]
pub enum Cli {
    Normal,
    /// `--restore-taskbar`: put the Windows taskbar back and exit.
    RestoreTaskbar,
    /// `--guard <pid>`: wait for that MacDows process to end, then restore the taskbar if it is still hidden.
    Guard(u32),
}

pub fn parse(args: &[&str]) -> Cli {
    match args {
        ["--restore-taskbar", ..] => Cli::RestoreTaskbar,
        ["--guard", pid, ..] => pid.parse().map(Cli::Guard).unwrap_or(Cli::Normal),
        _ => Cli::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_flags_are_recognized() {
        assert_eq!(parse(&["--restore-taskbar"]), Cli::RestoreTaskbar);
        assert_eq!(parse(&["--guard", "123"]), Cli::Guard(123));
    }

    #[test]
    fn anything_else_starts_the_app_normally() {
        assert_eq!(parse(&[]), Cli::Normal);
        assert_eq!(parse(&["--guard", "x"]), Cli::Normal);
        assert_eq!(parse(&["--guard"]), Cli::Normal);
        assert_eq!(parse(&["--something-else"]), Cli::Normal);
    }
}
