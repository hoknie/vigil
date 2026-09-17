use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};

use crate::ui::Look;

pub const HERE: &str = "\u{25b8}";

pub fn render(
    look: Look,
    outer: Rect,
    inner: Rect,
    rows: Vec<Line<'static>>,
    at: usize,
    buffer: &mut Buffer,
) -> Vec<(usize, Rect)> {
    let page = inner.height as usize;
    if page == 0 || rows.is_empty() {
        return Vec::new();
    }
    let count = rows.len();
    let top = at.saturating_sub(page - 1).min(count.saturating_sub(page));
    let mut drawn = Vec::new();

    for (index, row) in rows.into_iter().enumerate().skip(top).take(page) {
        let area = Rect {
            y: inner.y + (index - top) as u16,
            height: 1,
            ..inner
        };
        let here = index == at;
        let mut spans = vec![Span::raw(match here {
            true => HERE,
            false => " ",
        })];
        spans.extend(row.spans);
        let line = Line::from(spans);
        let paragraph = match here {
            true => Paragraph::new(line).style(look.palette.selected()),
            false => Paragraph::new(line),
        };
        paragraph.render(area, buffer);
        drawn.push((index, area));
    }

    if count > page {
        let mut state = ScrollbarState::new(count.saturating_sub(page)).position(top);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .render(
                Rect {
                    y: inner.y,
                    height: inner.height,
                    ..outer
                },
                buffer,
                &mut state,
            );
    }
    drawn
}
