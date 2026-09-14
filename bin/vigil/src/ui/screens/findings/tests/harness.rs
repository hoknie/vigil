use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::Severity;

use crate::ui::helpers::words::text;
use crate::ui::screens::findings::{Showing, render};
use crate::ui::{Dismissed, Filter, Picked, Sorting, View, fixture};

pub(super) static NOTHING_PICKED: Picked = Picked::none();

pub(super) static NOTHING_DISMISSED: Dismissed = Dismissed::none();

pub(super) fn showing(filter: &Filter) -> Showing<'_> {
    picking(filter, &NOTHING_PICKED, &NOTHING_DISMISSED)
}

pub(super) fn picking<'a>(
    filter: &'a Filter,
    picked: &'a Picked,
    dismissed: &'a Dismissed,
) -> Showing<'a> {
    Showing {
        filter,
        cursor: 0,
        focused: true,
        sorting: Sorting::default(),
        picked,
        dismissed,
    }
}

pub(super) fn drawn(view: &View, filter: &Filter, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 20));
    render(
        view,
        fixture::look(),
        &showing(filter),
        buffer.area,
        &mut buffer,
    );
    text::to_text(&buffer)
}

pub(super) fn floored(levels: usize) -> Filter {
    let mut filter = Filter::default();
    filter.set_floor(Severity::KNOWN[levels].clone());
    filter
}

pub(super) fn searching_for(wanted: &str) -> Filter {
    let mut filter = Filter::default();
    filter.search_mut().start();
    for character in wanted.chars() {
        filter.search_mut().type_character(character);
    }
    filter.search_mut().accept();
    filter
}

pub(super) fn storing(store: vigil_model::StoreStatus) -> View {
    let mut view = fixture::view();
    view.found.findings.clear();
    if let Some(status) = view.status.as_mut() {
        status.agent.findings.total = 0;
        status.agent.store = Some(store);
    }
    view
}
