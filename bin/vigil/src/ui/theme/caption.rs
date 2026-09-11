use ratatui::text::{Line, Span};

use crate::ui::Look;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keys {
    Sole,
    Here,
    Elsewhere,
}

pub fn render(look: Look, name: &str, tail: &str, keys: Keys, width: usize) -> Line<'static> {
    let marker = match keys {
        Keys::Here => "▸ ",
        _ => "  ",
    };
    let name = format!("{marker}{name} ");
    let tail = match tail.is_empty() {
        true => String::new(),
        false => format!(" {tail} "),
    };

    let used = name.chars().count() + tail.chars().count();
    let rule = "─".repeat(width.saturating_sub(used));

    Line::from(vec![
        Span::styled(
            name,
            match keys {
                Keys::Here => look.palette.selected(),
                _ => look.palette.heading(),
            },
        ),
        Span::styled(rule, look.palette.border()),
        Span::styled(tail, look.palette.quiet()),
    ])
}

pub fn scrolled(top: usize, page: usize, total: usize) -> String {
    if page == 0 {
        return String::new();
    }
    if total <= page {
        return format!("all {total} lines");
    }
    format!("lines {}-{} of {total}", top + 1, (top + page).min(total))
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(name: &str, tail: &str, keys: Keys, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 1));
        Paragraph::new(render(fixture::look(), name, tail, keys, width as usize))
            .render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn where_the_arrows_are_is_readable_with_no_colour_at_all() {
        let here = drawn("FINDINGS", "", Keys::Here, 40);
        let elsewhere = drawn("FINDINGS", "", Keys::Elsewhere, 40);

        assert!(here.starts_with("▸ FINDINGS"), "{here}");
        assert!(elsewhere.starts_with("  FINDINGS"), "{elsewhere}");
        assert_ne!(here, elsewhere, "the two panes must not read the same");
    }

    #[test]
    fn a_screen_with_one_pane_on_it_does_not_ask_which_pane_has_the_keys() {
        assert_eq!(
            drawn("THE SELECTED FINDING", "", Keys::Sole, 40),
            drawn("THE SELECTED FINDING", "", Keys::Elsewhere, 40)
        );
    }

    #[test]
    fn a_pane_with_more_below_the_fold_says_so_and_one_without_says_that_too() {
        assert_eq!(scrolled(0, 18, 42), "lines 1-18 of 42");
        assert_eq!(scrolled(24, 18, 42), "lines 25-42 of 42");
        assert_eq!(scrolled(0, 18, 12), "all 12 lines");
        assert_eq!(scrolled(0, 0, 12), "", "a window with no room says nothing");

        let page = drawn("THE SELECTED FINDING", &scrolled(0, 18, 42), Keys::Here, 60);
        assert!(page.contains("lines 1-18 of 42"), "{page}");
    }

    #[test]
    fn a_caption_wider_than_its_pane_keeps_the_words_and_loses_the_rule() {
        for width in [12u16, 24, 40, 80] {
            let page = drawn("THE SELECTED FINDING", "all 12 lines", Keys::Here, width);
            assert!(
                page.chars().count() <= width as usize,
                "{width}: {page} ({} columns)",
                page.chars().count()
            );
        }
    }
}
