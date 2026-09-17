use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use vigil_view::Piece;

use crate::ui::app::WATCHING;
use crate::ui::details::pieces;
use crate::ui::{Look, Target};

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
) -> Vec<(Rect, Target)> {
    self::pieces::framed(
        self::pieces::Framed {
            caption: CAPTION,
            buttons: Line::from(vec![
                Span::raw(" "),
                Span::styled("[ \u{2190} Back ]", look.palette.selected()),
                Span::raw("  "),
                Span::styled(counting(watching), look.palette.accent()),
            ]),
            pieces,
            top,
        },
        look,
        area,
        buffer,
    )
    .into_iter()
    .zip([Target::Back, Target::Counting])
    .collect()
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
