use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{
    Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
};
use vigil_view::Piece;

use super::report::report;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::{Look, Notice};

pub fn render(
    pieces: &[Piece],
    acts: Acts,
    at: Option<usize>,
    look: Look,
    top: usize,
    area: Rect,
    buffer: &mut Buffer,
) -> Vec<(usize, Rect)> {
    if pieces.is_empty() {
        Notice::plain("Nothing is selected.")
            .saying("Move to a row and press →. Esc closes this.")
            .render(look, area, buffer);
        return Vec::new();
    }

    let gutter = area.width - look.text_width(area.width) as u16;
    let report = report(pieces, acts, at, look, look.text_width(area.width));
    let page = area.height as usize;
    let top = top.min(report.len().saturating_sub(page));

    Paragraph::new(
        report
            .lines()
            .iter()
            .skip(top)
            .take(page)
            .cloned()
            .collect::<Vec<Line>>(),
    )
    .render(
        Rect {
            width: area.width - gutter,
            ..area
        },
        buffer,
    );

    if gutter > 0 && report.len() > page {
        let mut bar = ScrollbarState::new(report.len() - page).position(top);
        StatefulWidget::render(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .style(look.palette.border())
                .begin_symbol(None)
                .end_symbol(None),
            area,
            buffer,
            &mut bar,
        );
    }

    report
        .buttons()
        .iter()
        .filter(|(_, line, _, _)| (top..top + page).contains(line))
        .map(|(button, line, x, wide)| {
            let drawn = Rect::new(area.x + x, area.y + (line - top) as u16, *wide, 1);
            (*button, drawn.intersection(area))
        })
        .collect()
}

pub fn height(pieces: &[Piece], acts: Acts, look: Look, width: usize) -> usize {
    match pieces.is_empty() {
        true => 0,
        false => report(pieces, acts, None, look, width).len(),
    }
}
