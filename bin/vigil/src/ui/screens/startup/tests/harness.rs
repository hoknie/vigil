use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::startup::{Showing, render};
use crate::ui::{Arrows, Nesting, Search, Startup, View, fixture};

pub(super) fn drawn(view: &View, list: Startup, width: u16) -> String {
    drawn_as(
        view,
        showing(list, &Search::default(), Nesting::default()),
        width,
    )
}

pub(super) fn showing<'a>(list: Startup, search: &'a Search, nesting: Nesting) -> Showing<'a> {
    Showing {
        list,
        search,
        nesting,
        cursor: 0,
        elsewhere: 0,
        arrows: Arrows::List,
    }
}

pub(super) fn drawn_as(view: &View, showing: Showing<'_>, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 30));
    render(view, fixture::look(), &showing, buffer.area, &mut buffer);
    text::to_text(&buffer)
}
