use std::fmt;

use ratatui::crossterm::Command;

pub const CLICKS_ONLY: &str = "\x1b[?1000h\x1b[?1006h";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClicksOnly;

impl Command for ClicksOnly {
    fn write_ansi(&self, out: &mut impl fmt::Write) -> fmt::Result {
        out.write_str(CLICKS_ONLY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written() -> String {
        let mut out = String::new();
        ClicksOnly.write_ansi(&mut out).expect("a string takes it");
        out
    }

    #[test]
    fn the_console_asks_the_terminal_for_presses_and_never_for_movement() {
        assert_eq!(written(), "\u{1b}[?1000h\u{1b}[?1006h");
        assert!(
            !written().contains("?1003h") && !written().contains("?1002h"),
            "tracking movement redraws the whole screen every time the pointer crosses a cell, \
             and the console has nothing to show for it: {}",
            written().escape_debug()
        );
        assert!(
            written().contains("?1006h"),
            "without the long report a click past column 223 arrives as a click somewhere else"
        );
    }
}
