use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::startup::{Showing, render};
use crate::ui::{Arrows, Search, Startup, View, fixture};

pub(super) fn drawn(view: &View, list: Startup, width: u16) -> String {
    let search = Search::default();
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 30));
    render(
        view,
        fixture::look(),
        &Showing {
            list,
            search: &search,
            cursor: 0,
            elsewhere: 0,
            arrows: Arrows::List,
        },
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}
