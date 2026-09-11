use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::home::{Showing, render};
use crate::ui::{Arrows, View, fixture};

pub(super) fn drawn(view: &View, width: u16, height: u16) -> String {
    drawn_at(view, 0, width, height)
}

pub(super) fn drawn_at(view: &View, cursor: usize, width: u16, height: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, height));
    render(
        view,
        fixture::look(),
        &Showing {
            cursor,
            arrows: Arrows::List,
        },
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}
