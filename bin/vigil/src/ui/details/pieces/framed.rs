use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Widget};
use vigil_view::Piece;

use super::render::{height, render};
use crate::ui::Look;
use crate::ui::helpers::finding::acts::Acts;
use crate::ui::helpers::layout::split;
use crate::ui::theme::caption::{self, Keys};
use crate::ui::theme::panel;

pub struct Framed<'a> {
    pub caption: &'a str,
    pub buttons: Line<'static>,
    pub pieces: &'a [Piece],
    pub top: usize,
}

pub fn drawing(area: Rect) -> Rect {
    let inside = split::inside(area);
    Rect {
        y: inside.y + inside.height.min(2),
        height: inside.height.saturating_sub(2),
        ..inside
    }
}

pub fn framed(shown: Framed<'_>, look: Look, area: Rect, buffer: &mut Buffer) -> Vec<Rect> {
    let below = drawing(area);
    if below.height == 0 {
        return Vec::new();
    }
    let total = height(
        shown.pieces,
        Acts::default(),
        look,
        look.text_width(below.width),
    );
    let page = below.height as usize;

    panel::tail(
        panel::block(look, shown.caption, Keys::Here, true),
        look,
        &caption::scrolled(shown.top.min(total.saturating_sub(page)), page, total),
    )
    .render(area, buffer);
    let row = Rect {
        height: 1,
        ..split::inside(area)
    };
    let pressed = buttons(&shown.buttons, row);
    Paragraph::new(shown.buttons).render(row, buffer);
    render(
        shown.pieces,
        Acts::default(),
        None,
        look,
        shown.top,
        below,
        buffer,
    );
    pressed
}

fn buttons(line: &Line<'static>, row: Rect) -> Vec<Rect> {
    let mut x = row.x;
    let mut found = Vec::new();
    for span in &line.spans {
        let wide = span.width() as u16;
        if span.content.starts_with('[') {
            found.push(Rect::new(x, row.y, wide, 1).intersection(row));
        }
        x = x.saturating_add(wide);
    }
    found
}
