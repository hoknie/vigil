use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Widget};

use crate::ui::Look;
use crate::ui::helpers::layout::column;
pub fn render(look: Look, area: Rect, buffer: &mut Buffer) {
    let rows = rows();
    let width = 72.min(area.width);
    let height = (rows.len() as u16 + 2).min(area.height);
    let panel = Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height.saturating_sub(height)) / 2,
        width,
        height,
    };
    Clear.render(panel, buffer);

    let block = Block::bordered()
        .border_style(look.palette.border())
        .title(Line::styled(" KEYS ", look.palette.heading()))
        .title_bottom(
            Line::styled(" any key closes this ", look.palette.quiet()).alignment(Alignment::Right),
        );
    let inside = block.inner(panel);
    block.render(panel, buffer);

    Paragraph::new(
        rows.into_iter()
            .map(|(keys, meaning)| match keys.is_empty() {
                true => Line::styled(meaning.to_string(), look.palette.heading()),
                false => Line::from(vec![
                    Span::styled(
                        format!("  {}", column::fit(keys, 17)),
                        look.palette.accent(),
                    ),
                    Span::raw(meaning.to_string()),
                ]),
            })
            .collect::<Vec<Line>>(),
    )
    .render(inside, buffer);
}
fn rows() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "",
            "EVERY RUNG: → in, ← out; ▸ in a heading has the arrows now",
        ),
        ("1 - 9", "open one; the numbers are on the main screen"),
        ("", "IN A LIST OR A REPORT"),
        ("j / k, ↑ ↓", "a row at a time"),
        ("PgUp / PgDn", "a screenful"),
        ("g / G, Home/End", "the top / the end"),
        ("→ or Enter", "the detail of the row (not on the summary)"),
        ("/", "search: every value the agent read about a row"),
        ("o", "the object a finding is about; Esc comes back to it"),
        ("s / f", "sort this list / narrow the findings"),
        (
            "shift/ctrl ↑↓",
            "pick findings; x here, a all, d silence them, u undo",
        ),
        (
            "t T u U x, a",
            "show / hide kinds of socket, or all (ports)",
        ),
        (
            "t / d",
            "units as a tree (startup); a section's words (main)",
        ),
        ("", "THE LISTS OF A SECTION: ← → along them, ↓ into one"),
        ("", "  ports: sockets · by program"),
        (
            "",
            "  accounts: users · groups · sudo · keys · ssh users · logged in",
        ),
        ("", "  programs: running · launches"),
        ("", "  startup: units · timers · cron · modules · files"),
        ("", "  system: the host · watched files"),
        ("", "EVERYWHERE"),
        (
            "← or Esc",
            "a search, the detail, the panel, a rung, the main screen",
        ),
        ("r / ? / q", "ask now / this list / leave"),
    ]
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::Paragraph as Under;

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    #[test]
    fn every_key_the_console_uses_is_in_here() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

        render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        for key in [
            "1 - 9",
            "PgUp",
            "Enter",
            "o",
            "/",
            "s / f",
            "shift/ctrl",
            "d silence them",
            "Esc",
            "r",
            "as a tree",
            "a section's words",
        ] {
            assert!(page.contains(key), "{key} is not on the list: {page}");
        }
        assert!(
            !page.contains("Tab"),
            "a key the console ignores must not be offered: {page}"
        );
        assert!(
            page.contains("the numbers are on the main screen"),
            "a number is read off the main screen, not remembered: {page}"
        );
        assert!(page.contains("comes back to it"), "{page}");
        assert!(page.contains("any key closes this"), "{page}");
    }

    #[test]
    fn the_two_keys_that_go_back_are_one_line_because_they_mean_the_same_thing() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));

        render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        assert!(page.contains("← or Esc"), "{page}");
        assert!(
            page.contains("a search, the detail, the panel, a rung"),
            "the order of what one press undoes is the thing to look up: {page}"
        );
        assert!(page.contains("the main screen"), "{page}");
        assert!(
            !page.contains("closes the panel"),
            "the two keys were told apart and are not any more: {page}"
        );
    }

    #[test]
    fn what_was_underneath_does_not_show_through_it() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
        Under::new(vec!["xxxxxxxxxxxx".into(); 24]).render(buffer.area, &mut buffer);

        render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        let middle = page.lines().nth(12).expect("a line through the panel");
        let inside = middle
            .split('│')
            .nth(1)
            .expect("the panel has two sides to it");
        assert!(
            !inside.contains("xxx"),
            "the page underneath is showing through: {middle}"
        );
    }

    #[test]
    fn the_whole_list_fits_the_smallest_terminal_this_console_supports() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));

        render(fixture::look(), buffer.area, &mut buffer);

        let page = text::to_text(&buffer);
        assert!(page.contains("? / q"), "the last group is cut off: {page}");
        assert!(page.contains("startup:"), "{page}");
    }

    #[test]
    fn it_fits_the_terminal_it_is_opened_on() {
        for (width, height) in [(80u16, 24u16), (120, 40), (40, 12)] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
            render(fixture::look(), buffer.area, &mut buffer);

            for line in text::to_text(&buffer).lines() {
                assert!(line.chars().count() <= width as usize, "{width}: {line}");
            }
        }
    }
}
