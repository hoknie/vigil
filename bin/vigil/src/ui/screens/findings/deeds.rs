use ratatui::text::{Line, Span};

use crate::ui::{Deed, Look, Picked};

pub(super) fn bar(picked: &Picked, look: Look, width: u16) -> Line<'static> {
    let long = spans(picked, look, false);
    match drawn(&long) <= width as usize {
        true => Line::from(long),
        false => Line::from(spans(picked, look, true)),
    }
}

fn spans(picked: &Picked, look: Look, brief: bool) -> Vec<Span<'static>> {
    let mut spans = vec![Span::styled(
        format!(" {} picked ", picked.count()),
        look.palette.selected(),
    )];

    for deed in Deed::ALL {
        spans.push(Span::raw(" "));
        spans.push(Span::styled(deed.key().to_string(), look.palette.accent()));
        spans.push(Span::raw(format!(
            " {} \u{b7}",
            match brief {
                true => deed.briefly(),
                false => deed.named(),
            }
        )));
    }
    spans.push(Span::styled(" a", look.palette.accent()));
    spans.push(Span::raw(
        match brief {
            true => " all \u{b7}",
            false => " pick every row \u{b7}",
        }
        .to_string(),
    ));
    spans.push(Span::styled(" Esc", look.palette.accent()));
    spans.push(Span::raw(
        match brief {
            true => " none",
            false => " pick none",
        }
        .to_string(),
    ));
    spans
}

fn drawn(spans: &[Span<'static>]) -> usize {
    spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum::<usize>()
}

#[cfg(test)]
mod tests {
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::{Paragraph, Widget};

    use super::*;
    use crate::ui::fixture;
    use crate::ui::helpers::words::text;

    fn drawn_at(count: usize, width: u16) -> String {
        let mut picked = Picked::default();
        for row in 0..count {
            picked.toggle(&format!("row {row}"));
        }
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 1));
        Paragraph::new(bar(&picked, fixture::look(), width)).render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn the_bar_says_how_many_are_picked_and_what_can_be_done_to_them() {
        let line = drawn_at(3, 80);

        assert!(line.contains("3 picked"), "{line}");
        assert!(line.contains("d silence them and take them off"), "{line}");
        assert!(line.contains("Esc"), "{line}");
    }

    #[test]
    fn every_deed_this_console_can_do_is_on_the_bar_beside_its_key() {
        let line = drawn_at(1, 80);

        for deed in Deed::ALL {
            assert!(
                line.contains(deed.named()),
                "{} is a deed nobody is told about: {line}",
                deed.named()
            );
            assert!(line.contains(deed.key()), "{line}");
        }
    }

    #[test]
    fn it_fits_the_narrowest_terminal_this_console_is_read_on_and_keeps_the_keys() {
        for width in [40u16, 60, 80, 200] {
            let line = drawn_at(12, width);

            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
            assert!(line.contains("12 picked"), "{width}: {line}");
            assert!(line.contains(" d "), "{width}: {line}");
        }
    }
}
