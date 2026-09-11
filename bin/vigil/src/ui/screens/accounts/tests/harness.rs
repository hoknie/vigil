use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use vigil_model::Snapshot;

use crate::ui::helpers::words::text as page;
use crate::ui::screens::accounts::{Kind, Showing, render};
use crate::ui::{Arrows, Reading, Search, Subject, View, fixture};

pub(super) fn drawn(view: &View, subject: Subject) -> String {
    drawn_at(view, subject, 80)
}

pub(super) fn drawn_at(view: &View, subject: Subject, width: u16) -> String {
    drawn_seeking(view, subject, &Search::default(), width)
}

pub(super) fn drawn_seeking(view: &View, subject: Subject, search: &Search, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 40));
    render(
        view,
        fixture::look(),
        &Showing {
            subject,
            search,
            cursor: 0,
            elsewhere: 0,
            arrows: Arrows::List,
        },
        buffer.area,
        &mut buffer,
    );
    page::to_text(&buffer)
}

pub(super) fn looking_for(wanted: &str) -> Search {
    let mut search = Search::default();
    search.start();
    for character in wanted.chars() {
        search.type_character(character);
    }
    search.accept();
    search
}

pub(super) fn strip_sessions(view: &mut View) {
    let Reading::Taken(snapshot) = view.reading("users") else {
        panic!("the fixture has a reading");
    };
    let mut kept = Snapshot::new("users", snapshot.taken_at.clone());
    for (key, item) in &snapshot.items {
        if !matches!(Kind::of(key), Kind::Session | Kind::SessionSource) {
            kept = kept.with(key.clone(), item.clone());
        }
    }
    view.readings.put("users", Reading::Taken(kept));
}

pub(super) fn degrade(view: &mut View, reason: &str) {
    let status = view.status.as_mut().expect("the fixture answered");
    let users = status
        .agent
        .collectors
        .iter_mut()
        .find(|collector| collector.name == "users")
        .expect("the fixture has a users collector");
    users.state = vigil_model::CollectorState::Degraded;
    users.reason = Some(reason.to_string());
}
