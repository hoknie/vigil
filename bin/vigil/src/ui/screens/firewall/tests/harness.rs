use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::{CollectorRefusal, CollectorState};

use crate::ui::helpers::words::text;
use crate::ui::screens::firewall::{Showing, render};
use crate::ui::{Arrows, Audience, Gone, Look, Reading, Refusal, Search, Sorting, View, fixture};

#[derive(Default)]
pub(super) struct Given {
    search: Search,
    pub(super) gone: Option<Gone>,
    pub(super) sorting: Sorting,
}

impl Given {
    pub(super) fn showing(&self, cursor: usize) -> Showing<'_> {
        Showing {
            search: &self.search,
            cursor,
            arrows: Arrows::List,
            gone: self.gone.as_ref(),
            sorting: self.sorting,
        }
    }
}

pub(super) fn opened_on(key: &str, title: &str) -> Given {
    let mut finding = fixture::finding(title, vigil_model::Severity::High);
    finding.kind = vigil_model::Kind::Known(vigil_model::KnownKind::FirewallRulesetFlushed);
    Given {
        gone: Some(Gone::of(&finding, key)),
        ..Given::default()
    }
}

pub(super) fn looking_for(wanted: &str) -> Given {
    let mut given = Given::default();
    given.search.start();
    for character in wanted.chars() {
        given.search.type_character(character);
    }
    given.search.accept();
    given
}

pub(super) fn drawn(view: &View, width: u16) -> String {
    drawn_with(view, &Given::default(), 0, width, fixture::look())
}

pub(super) fn without_colour(view: &View, width: u16) -> String {
    drawn_with(
        view,
        &Given::default(),
        0,
        width,
        Look::new(fixture::monochrome(), Audience::Person),
    )
}

pub(super) fn squashed(page: &str) -> String {
    page.chars()
        .filter(|letter| !letter.is_whitespace())
        .collect()
}

pub(super) fn drawn_with(
    view: &View,
    given: &Given,
    cursor: usize,
    width: u16,
    look: Look,
) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 30));
    render(view, look, &given.showing(cursor), buffer.area, &mut buffer);
    text::to_text(&buffer)
}

pub(super) fn holding(snapshot: vigil_model::Snapshot) -> View {
    let mut view = fixture::view();
    view.readings.put("firewall", Reading::Taken(snapshot));
    view
}

pub(super) fn refused(state: CollectorState, reason: &str) -> View {
    let mut view = fixture::view();
    view.readings.put(
        "firewall",
        Reading::Refused(Refusal::told(CollectorRefusal::new(state, reason))),
    );
    view
}
