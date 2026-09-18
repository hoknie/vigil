use vigil_model::{CollectorState, CollectorStatus};

use super::harness::{drawn, note};
use crate::ui::fixture::screen;
use crate::ui::screens::home::rows;
use crate::ui::{View, fixture};

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

    let page = drawn(&view, 160, 32);

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
        screen("firewall"),
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
