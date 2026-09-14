use ratatui::text::{Line, Span};

use crate::ui::Look;
use crate::ui::helpers::layout::section;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Acts {
    pub acting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub key: char,
    pub name: &'static str,
}

impl Acts {
    pub fn of_a_row(acting: bool) -> Acts {
        Acts { acting }
    }

    pub fn buttons(self) -> Vec<Button> {
        match self.acting {
            false => Vec::new(),
            true => vec![
                Button {
                    key: 'K',
                    name: "close this socket",
                },
                Button {
                    key: 'S',
                    name: "suppress it",
                },
            ],
        }
    }

    pub fn button(self, at: usize) -> Option<Button> {
        self.buttons().get(at).copied()
    }
}

pub fn lines(acts: Acts, at: Option<usize>, look: Look, width: usize) -> Vec<Line<'static>> {
    let buttons = acts.buttons();
    if buttons.is_empty() {
        return Vec::new();
    }

    let arrows = match at {
        Some(_) => " \u{25b8} ",
        None => "   ",
    };
    let room = width.saturating_sub(4);
    let mut lines = vec![section::rule(look, "ACTIONS", width)];
    let mut spans = vec![Span::raw(arrows)];
    let mut used = 0;

    for (index, button) in buttons.iter().enumerate() {
        let said = format!("{} {}", button.key, button.name);
        let drawn = match (at, at == Some(index)) {
            (Some(_), false) => format!("  {said}  "),
            _ => format!("[ {said} ]"),
        };
        let wanted = drawn.chars().count() + 1;
        if used > 0 && used + wanted > room {
            lines.push(Line::from(std::mem::take(&mut spans)));
            spans.push(Span::raw("   "));
            used = 0;
        }
        used += wanted;
        spans.push(Span::styled(
            drawn,
            match (at, at == Some(index)) {
                (Some(_), true) => look.palette.selected(),
                (Some(_), false) => look.palette.quiet(),
                (None, _) => look.palette.accent(),
            },
        ));
        spans.push(Span::raw(" "));
    }
    lines.push(Line::from(spans));
    lines
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn(acts: Acts, at: Option<usize>, width: u16) -> String {
        let said = lines(acts, at, fixture::look(), width as usize);
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, said.len().max(1) as u16));
        Paragraph::new(said).render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn the_panel_acts_on_the_one_record_it_is_showing_and_never_on_a_marked_set() {
        let page = drawn(Acts::of_a_row(true), None, 80);

        assert!(page.contains("[ K close this socket ]"), "{page}");
        assert!(page.contains("[ S suppress it ]"), "{page}");
        assert!(
            !page.contains("mark"),
            "marking belongs to the list, where a reader can see what is being gathered. \
             This panel is open on one row and acts on that row: {page}"
        );
    }

    #[test]
    fn a_row_nothing_can_be_done_to_draws_no_buttons_at_all() {
        assert!(
            lines(Acts::of_a_row(false), None, fixture::look(), 80).is_empty(),
            "a button that answers with a refusal is worse than no button"
        );
    }

    #[test]
    fn where_the_arrows_are_is_readable_on_a_terminal_with_no_colour_at_all() {
        let elsewhere = drawn(Acts::of_a_row(true), None, 80);
        let on_the_first = drawn(Acts::of_a_row(true), Some(0), 80);

        assert!(
            !elsewhere.contains('\u{25b8}'),
            "the arrows are somewhere else in the panel: {elsewhere}"
        );
        assert!(
            on_the_first.contains('\u{25b8}'),
            "and when they are here, the row says so: {on_the_first}"
        );
        assert!(
            on_the_first.contains("[ K close this socket ]")
                && !on_the_first.contains("[ S suppress it ]"),
            "one of them is chosen and the rest are not, and the brackets say which without \
             a colour: {on_the_first}"
        );
        assert!(
            elsewhere.contains("[ K close this socket ]")
                && elsewhere.contains("[ S suppress it ]"),
            "with the arrows elsewhere both are offers and neither is chosen: {elsewhere}"
        );
    }

    #[test]
    fn the_arrows_walk_the_buttons_by_index_and_stop_at_the_ones_that_are_there() {
        assert_eq!(Acts::of_a_row(true).button(0).expect("a button").key, 'K');
        assert_eq!(Acts::of_a_row(true).button(1).expect("a button").key, 'S');
        assert!(Acts::of_a_row(true).button(2).is_none());
        assert!(Acts::of_a_row(false).button(0).is_none());
    }

    #[test]
    fn a_button_row_wider_than_the_panel_wraps_instead_of_being_cut_off_mid_word() {
        for width in [24u16, 30, 40, 66, 80] {
            let page = drawn(Acts::of_a_row(true), None, width);

            assert!(
                page.contains("[ S suppress it ]"),
                "{width}: the last button was cut off the end: {page}"
            );
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width}: {} columns: {line}",
                    line.chars().count()
                );
            }
        }
    }
}
