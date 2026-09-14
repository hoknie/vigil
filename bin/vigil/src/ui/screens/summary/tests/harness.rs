use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::summary::render;
use crate::ui::{View, fixture};

pub(super) fn drawn(width: u16) -> String {
    drawn_from(&fixture::view(), width)
}

pub(super) fn drawn_from(view: &View, width: u16) -> String {
    saying(view, width, true)
}

pub(super) fn saying(view: &View, width: u16, saying: bool) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 60));
    render(
        view,
        fixture::look(),
        0,
        saying,
        None,
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

pub(super) fn with_a_collector_switched_off() -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.push(fixture::collector_off());
    }
    view
}

pub(super) fn with_a_store() -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.store = Some(fixture::store());
    }
    view
}

pub(super) fn row<'a>(page: &'a str, named: &str) -> &'a str {
    page.lines()
        .find(|line| line.contains(named))
        .unwrap_or_else(|| panic!("no row for {named}: {page}"))
}
