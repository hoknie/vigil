use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::chrome::frame::{Hints, render};
use crate::ui::helpers::words::text;
use crate::ui::{Look, Screen, View, fixture};

pub(super) fn quiet() -> Hints<'static> {
    Hints::default()
}

pub(super) fn drawn(
    look: Look,
    screen: Screen,
    view: &View,
    width: u16,
    height: u16,
) -> (String, Rect) {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    let body = render(look, screen, view, quiet(), buffer.area, &mut buffer);
    (text::to_text(&buffer), body)
}

pub(super) fn drawn_body() -> Rect {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 100, 24));
    render(
        fixture::look(),
        Screen::FINDINGS,
        &fixture::view(),
        quiet(),
        buffer.area,
        &mut buffer,
    )
}
