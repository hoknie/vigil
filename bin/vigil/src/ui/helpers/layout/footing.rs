use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::Look;

pub fn onto_the_edge(look: Look, area: Rect, whole: Rect) -> Rect {
    match look.interactive() && area.bottom() < whole.bottom() && area.height > 0 {
        true => Rect {
            height: area.height + 1,
            ..area
        },
        false => area,
    }
}

pub fn render(look: Look, tally: String, area: Rect, buffer: &mut Buffer) {
    if area.height == 0 {
        return;
    }
    match look.interactive() {
        true => {
            Line::from(Span::styled(format!("{tally} "), look.palette.quiet())).render(area, buffer)
        }
        false => Paragraph::new(Line::styled(tally, look.palette.quiet())).render(area, buffer),
    }
}

#[cfg(test)]
mod tests {
    use ratatui::widgets::{Block, BorderType};

    use super::*;
    use crate::ui::helpers::words::text;
    use crate::ui::{Audience, fixture};

    #[test]
    fn a_tally_written_on_the_bottom_edge_keeps_the_rest_of_the_edge_drawn() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 4));
        let block = Block::bordered().border_type(BorderType::Rounded);
        let inside = block.inner(buffer.area);
        block.render(buffer.area, &mut buffer);

        let list = onto_the_edge(fixture::look(), inside, buffer.area);
        render(
            fixture::look(),
            " 12 row(s)".to_string(),
            Rect {
                y: list.bottom() - 1,
                height: 1,
                ..list
            },
            &mut buffer,
        );

        let page = text::to_text(&buffer);
        assert_eq!(
            page.lines().last(),
            Some(
                "\u{2570} 12 row(s) \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{256f}"
            ),
            "the tally is the footer of the frame, and the frame is still closed after it: {page}"
        );
    }

    #[test]
    fn a_list_with_no_frame_below_it_keeps_its_tally_inside_its_own_area() {
        let whole = Rect::new(0, 0, 40, 10);

        assert_eq!(
            onto_the_edge(fixture::look(), whole, whole),
            whole,
            "a terminal with no room for a frame has no edge to write on"
        );
        assert_eq!(
            onto_the_edge(
                Look::new(fixture::monochrome(), Audience::Script),
                Rect::new(0, 3, 40, 5),
                whole
            ),
            Rect::new(0, 3, 40, 5),
            "a page printed for a script has no frame, so its tally stays a line of text"
        );
    }
}
