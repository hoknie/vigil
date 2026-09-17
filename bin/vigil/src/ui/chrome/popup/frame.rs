use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Widget};

use crate::ui::Look;

pub fn render(
    look: Look,
    area: Rect,
    title: Line<'static>,
    footing: Option<Line<'static>>,
    buffer: &mut Buffer,
) -> Rect {
    Clear.render(area, buffer);
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(look.palette.border())
        .title_top(title)
        .shadow(look.palette.shadow());
    if let Some(footing) = footing {
        block = block.title_bottom(footing);
    }
    let inner = block.inner(area);
    block.render(area, buffer);
    inner
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Modifier};
    use ratatui::widgets::Paragraph;

    use super::*;
    use crate::ui::helpers::words::text;
    use crate::ui::{Audience, Palette, fixture};

    const UNDER: &str = "the text beside and below a popup is still the host's text";

    fn over_a_page(look: Look) -> Buffer {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 60, 8));
        Paragraph::new(vec![Line::raw(UNDER); 8]).render(buffer.area, &mut buffer);
        render(
            look,
            Rect::new(2, 1, 20, 4),
            Line::raw(" sort by "),
            None,
            &mut buffer,
        );
        buffer
    }

    fn shadowed() -> Vec<(u16, u16)> {
        let mut cells: Vec<(u16, u16)> = (3..23).map(|x| (x, 5)).collect();
        cells.extend((2..5).map(|y| (22, y)));
        cells
    }

    #[test]
    fn with_colour_the_shadow_dims_the_cells_under_it_and_leaves_their_text_readable() {
        let look = Look::new(
            Palette::decide(None, Some("xterm-256color")),
            Audience::Person,
        );
        let buffer = over_a_page(look);
        let page = text::to_text(&buffer);

        assert!(
            !page.contains('\u{2591}'),
            "a terminal with colour can dim a cell, so the shadow does not hide the text: {page}"
        );
        assert!(
            page.lines()
                .nth(5)
                .is_some_and(|line| line.starts_with("th") && line.contains("beside and below")),
            "the line under the popup reads as it did: {page}"
        );
        for (x, y) in shadowed() {
            let cell = &buffer[(x, y)];
            assert!(
                cell.modifier.contains(Modifier::DIM),
                "{x},{y} {:?} is under the shadow and is not dimmed: {page}",
                cell.symbol()
            );
            assert_eq!(
                (cell.fg, cell.bg),
                (Color::Reset, Color::Reset),
                "{x},{y}: dimming is the shadow, a painted background would be the terminal's \
                 theme overruled"
            );
        }
        assert!(
            !buffer[(40, 6)].modifier.contains(Modifier::DIM),
            "and nothing outside the shadow is dimmed"
        );
    }

    #[test]
    fn with_no_colour_the_shadow_is_the_light_shade_so_it_is_still_seen() {
        let buffer = over_a_page(fixture::look());
        let page = text::to_text(&buffer);

        for (x, y) in [(3u16, 5u16), (12, 5), (22, 2), (22, 5)] {
            assert_eq!(
                buffer[(x, y)].symbol(),
                "\u{2591}",
                "{x},{y}: a monochrome terminal has no dimming to show, so the shadow is drawn \
                 in characters: {page}"
            );
        }
    }
}
