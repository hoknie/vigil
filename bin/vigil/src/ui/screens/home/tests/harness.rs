use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::{CollectorState, CollectorStatus};

use crate::ui::helpers::words::text;
use crate::ui::screens::home::{Showing, render, rows};
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

pub(super) fn without_a_collector(name: &str) -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.retain(|it| it.name != name);
    }
    view
}

pub(super) fn note(view: &View, section: &str) -> Option<String> {
    rows(view)
        .into_iter()
        .find(|row| row.name == section)
        .and_then(|row| row.standing.note)
}

pub(super) fn without_a_section() -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.push(CollectorStatus {
            name: "network".into(),
            state: CollectorState::Ok,
            reason: None,
            items: 9,
            ..fixture::collector_off()
        });
    }
    view
}
