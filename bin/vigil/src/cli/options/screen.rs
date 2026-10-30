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
        .ok_or_else(|| format!("\n{}", wrapped(&screen_names(), "  ", IN_AN_ERROR)))
}

pub(super) fn help_for_screen() -> String {
    format!(
        "Which screen to open it on.\n\n{}\n\n`vigil ui` opens on home, `vigil capture` prints summary.",
        wrapped(&screen_names(), "", IN_THE_HELP)
    )
}

const DIFFERENCE: &str = "difference";

const IN_AN_ERROR: usize = 78;

const IN_THE_HELP: usize = 68;

fn screen_names() -> String {
    let sections: Vec<&str> = Screen::all()
        .into_iter()
        .filter(|screen| *screen != Screen::SUMMARY && *screen != Screen::FINDINGS)
        .map(Screen::name)
        .collect();

    format!(
        "the sections are {}; `summary` and `findings` are the agent's own, and `home` is \
         the screen that lists them (`{DIFFERENCE}` too: the findings with the detail panel \
         already open)",
        sections.join(", ")
    )
}

fn wrapped(text: &str, indent: &str, width: usize) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && indent.len() + line.len() + 1 + word.len() > width {
            lines.push(format!("{indent}{line}"));
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(format!("{indent}{line}"));
    }

    lines.join("\n")
}
