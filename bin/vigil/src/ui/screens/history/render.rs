use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};

use crate::ui::details::pieces;
use crate::ui::{History, Look, Target};

pub const CAPTION: &str = "THE HISTORY OF THE SELECTED ROW";

pub fn render(
    history: &History,
    look: Look,
    area: Rect,
    buffer: &mut Buffer,
) -> Vec<(Rect, Target)> {
    pieces::framed(
        pieces::Framed {
            caption: CAPTION,
            buttons: Line::from(vec![
                Span::raw(" "),
                Span::styled("[ \u{2190} Back ]", look.palette.selected()),
            ]),
            pieces: history.pieces(),
            top: history.top(),
        },
        look,
        area,
        buffer,
    )
    .into_iter()
    .map(|drawn| (drawn, Target::Back))
    .collect()
}
