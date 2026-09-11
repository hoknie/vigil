use ratatui::text::{Line, Span};

use crate::ui::Look;

pub fn rule(look: Look, name: &str, width: usize) -> Line<'static> {
    let name = format!(" {name} ");
    let used = name.chars().count();
    let trailing = width.saturating_sub(used);

    Line::from(vec![
        Span::styled(name, look.palette.heading()),
        Span::styled("─".repeat(trailing), look.palette.border()),
    ])
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(name: &str, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 1));
        Paragraph::new(rule(fixture::look(), name, width as usize))
            .render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn the_heading_is_readable_and_the_rule_reaches_the_edge() {
        let line = drawn("SUDO", 40);

        assert!(line.starts_with(" SUDO "), "{line}");
        assert_eq!(line.chars().count(), 40, "{line}");
        assert!(line.ends_with('─'), "{line}");
    }

    #[test]
    fn a_heading_wider_than_the_terminal_keeps_the_words_and_loses_the_rule() {
        let line = drawn("A HEADING LONGER THAN THE TERMINAL IT IS DRAWN IN", 20);

        assert!(line.starts_with(" A HEADING"), "{line}");
        assert!(!line.contains('─'), "{line}");
        assert!(line.chars().count() <= 20, "{line}");
    }
}
