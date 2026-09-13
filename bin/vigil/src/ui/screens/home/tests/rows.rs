use vigil_model::{CollectorState, CollectorStatus};

use super::harness::drawn;
use crate::ui::fixture::screen;
use crate::ui::screens::home::{keys, rows};
use crate::ui::{Screen, View, fixture};

fn without_a_collector(name: &str) -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.retain(|it| it.name != name);
    }
    view
}

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

fn note(view: &View, section: &str) -> Option<String> {
    rows(view)
        .into_iter()
        .find(|row| row.name == section)
        .and_then(|row| row.standing.note)
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
            .any(|row| row.name == "network" && row.opens.is_none()),
        "a reading with nowhere to open is a row and not a silence"
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

fn without_a_section() -> View {
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

#[test]
fn a_reading_that_is_incomplete_is_marked_and_carries_the_reason_the_agent_gave() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut()
        && let Some(persistence) = status
            .agent
            .collectors
            .iter_mut()
            .find(|it| it.name == "persistence")
    {
        persistence.state = CollectorState::Unavailable;
        persistence.reason = Some("/etc/cron.d cannot be read: not running as root".into());
    }

    let page = drawn(&view, 80, 30);

    assert!(
        page.lines()
            .any(|line| line.contains("startup") && line.contains('!')),
        "{page}"
    );
    assert!(
        !page.contains("/etc/cron.d cannot be read"),
        "the reason moved to the panel: {page}"
    );
    assert!(
        note(&view, "startup").is_some_and(|note| note.contains("/etc/cron.d cannot be read")),
        "and the row carries it for the panel to draw"
    );
    assert!(page.contains("less than asked for"), "{page}");
}

#[test]
fn the_summary_has_no_objects_and_no_moment_of_reading_because_it_is_not_a_reading() {
    let view = fixture::view();

    let summary = rows(&view)
        .into_iter()
        .find(|row| row.name == "summary")
        .expect("a row for the summary");

    assert_eq!(
        summary.standing.objects, None,
        "a number here would be read as a count of objects the agent holds"
    );
    assert_eq!(summary.standing.state, "answering");
}

#[test]
fn a_collector_this_console_has_a_section_for_is_never_a_row_with_nothing_behind_it() {
    let view = fixture::view();

    let firewall = rows(&view)
        .into_iter()
        .find(|row| row.name == "firewall")
        .expect("a row for the firewall");

    assert_eq!(
        firewall.opens,
        Some(screen("firewall")),
        "a reading with a section of its own must open it, not sit among the strangers"
    );
    assert_eq!(firewall.number, Some(5));
    assert_ne!(
        firewall.standing.note.as_deref(),
        Some(super::super::notices::NO_SECTION),
        "the sentence about a reading nobody drew a screen for belongs to a stranger, not to \
         a section that has one"
    );
    assert_eq!(firewall.collector, "firewall");
}

#[test]
fn a_buffer_that_is_dropping_findings_marks_the_main_screen_and_being_behind_does_not() {
    let mut behind = fixture::view();
    if let Some(status) = behind.status.as_mut() {
        status.agent.buffers = Some(vec![fixture::behind("ndjson")]);
    }
    let mut losing = fixture::view();
    if let Some(status) = losing.status.as_mut() {
        status.agent.buffers = Some(vec![fixture::losing("ndjson")]);
    }

    let standing = |view: &View| {
        rows(view)
            .into_iter()
            .find(|row| row.name == "summary")
            .expect("a row for the summary")
            .standing
    };

    assert!(
        !standing(&behind).unwell,
        "a receiver that is behind will be caught up, and a mark that cries at that is a mark \
         nobody reads by the third day"
    );
    assert!(
        standing(&losing).unwell,
        "findings the agent threw away are the one thing on this screen that has to be seen \
         without opening anything"
    );
    assert!(
        note(&losing, "summary").is_some_and(|note| note.contains("dropped at the ceiling")),
        "and the panel says how many and out of what"
    );
    assert!(
        drawn(&losing, 80, 30)
            .lines()
            .any(|line| line.contains("summary") && line.contains('!')),
        "{}",
        drawn(&losing, 80, 30)
    );
}

#[test]
fn a_collector_whose_name_is_longer_than_a_sections_is_drawn_whole_at_eighty_columns() {
    for name in [
        "network",
        "containers",
        "file-integrity",
        "everything-on-this-host",
    ] {
        let mut view = fixture::view();
        if let Some(status) = view.status.as_mut() {
            status.agent.collectors.push(CollectorStatus {
                name: name.into(),
                state: CollectorState::Ok,
                reason: None,
                items: 9,
                ..fixture::collector_off()
            });
        }

        let page = drawn(&view, 80, 40);

        assert!(
            page.lines().any(|line| line.contains(name)),
            "{name} is not on the main screen whole: {page}"
        );
        for line in page.lines() {
            assert!(line.chars().count() <= 80, "{name}: {line}");
            assert!(
                !line.contains('\u{2026}'),
                "the column is sized by the rows it draws, and this one was sized by the \
                 sections alone: {name}: {line}"
            );
        }
    }
}
