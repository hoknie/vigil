use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::ui::details::pieces;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::{History, Look};

pub const HEADER_LINES: u16 = 2;

pub const CAPTION: &str = "THE HISTORY OF THE SELECTED ROW";

pub fn render(history: &History, look: Look, area: Rect, buffer: &mut Buffer) {
    let header = HEADER_LINES.min(area.height);
    if header == 0 {
        return;
    }

    Paragraph::new(vec![
        Line::from(vec![
            Span::raw(" "),
            Span::styled("[ \u{2190} Back ]", look.palette.selected()),
            Span::raw("  "),
            Span::styled(CAPTION, look.palette.heading()),
        ]),
        Line::raw(""),
    ])
    .render(
        Rect {
            height: header,
            ..area
        },
        buffer,
    );

    let below = Rect {
        y: area.y + header,
        height: area.height - header,
        ..area
    };
    if below.height == 0 {
        return;
    }
    pieces::render(
        history.pieces(),
        Acts::default(),
        None,
        look,
        history.top(),
        below,
        buffer,
    );
}
