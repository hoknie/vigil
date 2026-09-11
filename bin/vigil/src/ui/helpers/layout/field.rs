use ratatui::text::{Line, Span};

use crate::ui::Look;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::wrap;
pub fn lines(
    look: Look,
    name: &str,
    value: &str,
    name_width: usize,
    width: usize,
) -> Vec<Line<'static>> {
    let indent = name_width + 4;
    wrap::wrap(value, width.saturating_sub(indent + 1))
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            Line::from(vec![
                Span::styled(
                    format!(
                        "   {}",
                        column::fit(
                            match index {
                                0 => name,
                                _ => "",
                            },
                            name_width
                        )
                    ),
                    look.palette.label(),
                ),
                Span::raw(format!(" {part}")),
            ])
        })
        .collect()
}
pub fn one(look: Look, name: &str, value: &str, name_width: usize, width: usize) -> Line<'static> {
    let indent = name_width + 4;
    Line::from(vec![
        Span::styled(
            format!("   {}", column::fit(name, name_width)),
            look.palette.label(),
        ),
        Span::raw(format!(
            " {}",
            column::fit(value, width.saturating_sub(indent))
        )),
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

    fn drawn(name: &str, value: &str, width: u16) -> String {
        let mut buffer = Buffer::empty(Rect::new(0, 0, width, 6));
        Paragraph::new(lines(fixture::look(), name, value, 10, width as usize))
            .render(buffer.area, &mut buffer);
        text::to_text(&buffer)
    }

    #[test]
    fn the_name_sits_in_a_column_of_its_own_and_the_value_beside_it() {
        let page = drawn("host id", "1c9d8e7b4a5c6d0e", 60);

        assert_eq!(page, "   host id    1c9d8e7b4a5c6d0e");
    }

    #[test]
    fn a_value_wider_than_the_line_is_wrapped_under_itself_and_not_under_the_name() {
        let page = drawn(
            "key",
            "SHA256:aaaaaaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbbbb",
            40,
        );

        let lines: Vec<&str> = page.lines().collect();
        assert_eq!(lines.len(), 2, "{page}");
        assert!(lines[0].starts_with("   key        SHA256:"), "{page}");
        assert!(
            lines[1].starts_with("              ") && !lines[1].trim_start().is_empty(),
            "the second line lines up under the value, not under the name: {page}"
        );
        for line in lines {
            assert!(line.chars().count() <= 40, "{line}");
        }
    }

    #[test]
    fn without_colour_it_is_exactly_the_text_it_always_was() {
        assert_eq!(
            drawn("host", "app-01", 40),
            "   host       app-01",
            "a monochrome terminal must read the same as it did before labels were coloured"
        );
    }
}
