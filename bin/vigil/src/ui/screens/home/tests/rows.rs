use vigil_model::{CollectorState, CollectorStatus};

use super::harness::{drawn, note, without_a_collector, without_a_section};
use crate::ui::screens::home::{keys, rows};
use crate::ui::{Screen, fixture};

#[test]
fn every_section_of_this_console_is_a_row_and_the_reader_can_reach_it() {
    let view = fixture::view();

    let named: Vec<String> = keys(&view);

    for screen in Screen::all() {
        assert!(
            named.iter().any(|name| name == screen.name()),
            "{} is unreachable: a section that is not on the main screen does not exist",
            screen.name()
        );
    }
}

#[test]
fn a_section_whose_collector_is_switched_off_is_a_row_with_the_reason_on_it() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.retain(|it| it.name != "processes");
        status.agent.collectors.retain(|it| it.name != "launches");
        status.agent.collectors.push(CollectorStatus {
            state: CollectorState::Off,
            ..fixture::collector_off()
        });
        status.agent.collectors.push(CollectorStatus {
            name: "processes".into(),
            state: CollectorState::Off,
            reason: Some("not named in `collectors:`".into()),
            ..fixture::collector_off()
        });
    }

    let page = drawn(&view, 80, 30);

    let row = page
        .lines()
        .find(|line| line.contains("programs"))
        .expect("the section is still a row");
    assert!(row.contains("off"), "{row}");
    assert!(
        row.contains('—'),
        "a section that is off has no count and no moment of reading: {row}"
    );
    assert!(
        !page.contains("collectors:"),
        "the reason belongs in the panel beside the list, not in a sentence wrapped under \
         the row: a table with prose between its rows cannot be read down a column: {page}"
    );
    assert!(
        note(&view, "programs").is_some_and(|note| note.contains("not named in `collectors:`")),
        "and the row still carries it, for the panel to draw — both collectors of it, named"
    );
}

#[test]
fn a_collector_this_console_has_no_section_for_is_still_a_row() {
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

    let page = drawn(&view, 80, 30);

    assert!(page.contains("network"), "{page}");
    assert!(
        !page.contains("No screen in this console draws"),
        "the sentence moved to the panel: {page}"
    );
    assert!(
        note(&view, "network").is_some_and(|note| note.contains("No screen in this console draws")),
        "and the row still carries it, for the panel to draw"
    );
    assert!(
        note(&view, "network").is_some_and(|note| !note.contains("newer")),
        "the console and the agent ship in one package, so a reading nobody drew a screen for \
         is not an agent that ran ahead"
    );
    assert!(
        rows(&view)
            .iter()
            .any(|row| row.name == "network" && row.opens == Screen::UNKNOWN),
        "a reading this console has no screen for opens as a plain list: accepting what it \
         does not know beats drawing nothing"
    );
    assert!(
        !rows(&view)
            .iter()
            .any(|row| row.name == "network" && row.is_a_section_of_its_own()),
        "and it is still counted apart from the sections this build has screens for"
    );
}

#[test]
fn a_collector_this_agent_does_not_watch_reads_differently_from_one_this_console_cannot_draw() {
    let view = without_a_collector("persistence");
    let page = drawn(&view, 80, 30);

    let row = page
        .lines()
        .find(|line| line.contains("startup"))
        .expect("the section is still a row");
    assert!(row.contains("not watched"), "{row}");
    assert_ne!(
        note(&view, "startup"),
        note(&without_a_section(), "network"),
        "an agent that never reads it and a console that cannot show it are two hosts \
         to go and look at, and the two sentences are what tells them apart"
    );
    assert!(
        note(&view, "startup").is_some_and(|note| note.contains("does not watch it")),
        "{page}"
    );
}
