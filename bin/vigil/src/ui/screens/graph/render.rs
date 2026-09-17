use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use vigil_view::Piece;

use crate::ui::Look;
use crate::ui::app::WATCHING;
use crate::ui::details::pieces;
use crate::ui::helpers::finding::acts::Acts;

pub const HEADER_LINES: u16 = 2;

pub const CAPTION: &str = "HOW A PACKET TRAVELS";

const START: &str = "start counting";

const STOP: &str = "stop counting";

pub fn render(
    pieces: &[Piece],
    watching: bool,
    look: Look,
    top: usize,
    area: Rect,
    buffer: &mut Buffer,
) {
    let header = HEADER_LINES.min(area.height);
    if header == 0 {
        return;
    }

    Paragraph::new(vec![
        Line::from(vec![
            Span::raw(" "),
            Span::styled("[ \u{2190} Back ]", look.palette.selected()),
            Span::raw("  "),
            Span::styled(counting(watching), look.palette.accent()),
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
    self::pieces::render(pieces, Acts::default(), None, look, top, below, buffer);
}

pub fn counting(watching: bool) -> String {
    format!(
        "[ {WATCHING} {} ]",
        match watching {
            true => STOP,
            false => START,
        }
    )
}
