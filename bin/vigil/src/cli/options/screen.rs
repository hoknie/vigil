use super::opening::Opening;
use crate::ui::Screen;

pub(super) fn screen(name: &str) -> Result<Opening, String> {
    if name == DIFFERENCE {
        return Ok(Opening {
            screen: Screen::FINDINGS,
            difference: true,
        });
    }
    Screen::parse(name)
        .map(|screen| Opening {
            screen,
            difference: false,
        })
        .ok_or_else(|| SCREEN_NAMES.to_string())
}

const DIFFERENCE: &str = "difference";

const SCREEN_NAMES: &str = "\
\n  the sections are ports, accounts, programs, startup, firewall, summary and\
\n  findings,\
\n  and `home` is the screen that lists them\
\n  (`difference` too: the findings with the detail panel already open)";
