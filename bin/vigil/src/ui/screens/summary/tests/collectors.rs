use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use super::harness::{drawn, drawn_from, saying, with_a_collector_switched_off};
use crate::ui::fixture;
use crate::ui::helpers::words::text;
use crate::ui::screens::summary::render;

#[test]
fn a_degraded_collector_says_why_next_to_itself() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].state = vigil_model::CollectorState::Degraded;
        status.agent.collectors[0].reason = Some("cannot resolve socket owners".into());
    }

    let page = drawn_from(&view, 80);

    assert!(page.contains("degraded"), "{page}");
    assert!(page.contains("cannot resolve socket owners"), "{page}");
}

#[test]
fn a_collector_switched_off_has_its_row_and_reads_as_a_decision() {
    let page = drawn_from(&with_a_collector_switched_off(), 80);

    let row = page
        .lines()
        .find(|line| line.contains("launches"))
        .unwrap_or_else(|| panic!("no row for the collector that is off: {page}"));
    assert!(row.contains("off"), "{row}");
    assert!(
        !row.contains("never") && !row.contains("not yet"),
        "a collector that is off is not one whose first reading is due: {row}"
    );
    let reason = fixture::collector_off()
        .reason
        .expect("a collector that is off names its cause");
    for word in reason.split_whitespace() {
        assert!(
            page.contains(word),
            "the reason the daemon gave is cut before {word}: {page}"
        );
    }
}

#[test]
fn the_row_that_is_off_is_told_apart_without_any_colour_at_all() {
    let page = drawn_from(&with_a_collector_switched_off(), 120);
    let lines: Vec<&str> = page.lines().collect();

    let off = lines
        .iter()
        .find(|line| line.contains("launches"))
        .expect("the row that is off");
    let reading = lines
        .iter()
        .find(|line| line.contains("network") && line.contains("ok"))
        .expect("a row that is reading");

    assert!(off.contains("off"), "{off}");
    assert!(!off.contains(" ok "), "{off}");
    assert_ne!(
        off.split_whitespace().collect::<Vec<_>>(),
        reading.split_whitespace().collect::<Vec<_>>()
    );
}

#[test]
fn a_collector_that_does_not_report_a_period_is_not_a_collector_with_a_period_of_zero() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].every_seconds = None;
    }

    let page = drawn_from(&view, 120);

    let row = page
        .lines()
        .find(|line| line.contains("network") && line.contains("ok"))
        .expect("the row for the collector that says nothing");
    assert!(
        row.contains("not reported"),
        "a period nobody stated is not a period of zero seconds: {row}"
    );
    assert!(
        !row.contains("every 0 s"),
        "and a silent zero here is the failure this product exists to refuse: {row}"
    );
}

#[test]
fn the_table_says_how_often_each_collector_reads_and_a_wide_one_says_when_it_reads_next() {
    let narrow = drawn(80);
    let wide = drawn(120);

    assert!(narrow.contains("EVERY"), "{narrow}");
    assert!(narrow.contains("every 30 s"), "{narrow}");
    assert!(narrow.contains("every 300 s"), "{narrow}");
    assert!(
        !narrow.contains("NEXT"),
        "no room for it at eighty: {narrow}"
    );
    assert!(wide.contains("NEXT"), "{wide}");
    assert!(wide.contains("09:00:11"), "{wide}");
}

#[test]
fn a_reading_the_host_slept_through_is_a_number_and_not_a_silence() {
    let page = drawn(80);

    assert!(
        page.contains("2 reading(s) dropped"),
        "the fixture has a collector that missed two slots: {page}"
    );
    assert!(page.contains("still running"), "{page}");
}

#[test]
fn a_wide_terminal_also_says_what_each_reading_cost_and_whether_it_has_a_baseline() {
    let wide = drawn(160);

    assert!(wide.contains("BASELINE"), "{wide}");
    assert!(wide.contains("5 ms"), "{wide}");
    assert!(!drawn(80).contains("BASELINE"), "not at eighty, though");
}

#[test]
fn a_reason_is_folded_away_until_it_is_asked_for_and_the_row_says_there_is_one() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].state = vigil_model::CollectorState::Degraded;
        status.agent.collectors[0].reason = Some("cannot resolve socket owners".into());
    }

    let folded = saying(&view, 80, false);
    let unfolded = saying(&view, 80, true);

    assert!(
        !folded.contains("cannot resolve socket owners"),
        "a table with prose wrapped between its rows cannot be read down a column: {folded}"
    );
    assert!(
        folded
            .lines()
            .any(|line| line.contains("network") && line.contains('!')),
        "the row has to say there is something to read, or folding it away hides it: {folded}"
    );
    assert!(
        unfolded.contains("cannot resolve socket owners"),
        "{unfolded}"
    );
}

#[test]
fn a_table_where_every_collector_is_well_offers_nothing_to_unfold() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        for collector in status.agent.collectors.iter_mut() {
            collector.reason = None;
            collector.last_error = None;
            collector.skipped = 0;
        }
    }

    let folded = saying(&view, 80, false);

    assert!(
        folded
            .lines()
            .filter(|line| line.contains("every 30 s"))
            .all(|line| !line.contains('!')),
        "{folded}"
    );
}

#[test]
fn the_mark_on_a_row_that_has_something_to_say_reads_the_same_with_no_colour() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].reason = Some("cannot resolve socket owners".into());
    }

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 60));
    render(
        &view,
        crate::ui::Look::new(fixture::monochrome(), crate::ui::Audience::Person),
        0,
        false,
        None,
        buffer.area,
        &mut buffer,
    );
    let plain = text::to_text(&buffer);

    assert_eq!(
        plain,
        saying(&view, 80, false),
        "the mark is a character before it is a colour"
    );
}

fn with(
    state: vigil_model::CollectorState,
    reason: Option<&str>,
    limit: Option<&str>,
) -> crate::ui::View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors[0].state = state;
        status.agent.collectors[0].reason = reason.map(str::to_string);
        status.agent.collectors[0].limit = limit.map(str::to_string);
    }
    view
}

#[test]
fn a_limited_collector_says_limited_and_is_marked_apart_from_a_degraded_one() {
    let view = with(
        vigil_model::CollectorState::Limited,
        None,
        Some("the sockets of every other account are not seen"),
    );

    let folded = saying(&view, 80, false);
    let row = folded
        .lines()
        .find(|line| line.contains("network"))
        .unwrap_or_else(|| panic!("no row for network: {folded}"));
    assert!(row.contains("limited"), "{row}");
    assert!(row.contains('~'), "{row}");
    assert!(
        !row.contains('!'),
        "a limit of the mode is marked apart from a failure, and without colour: {row}"
    );

    let unfolded = saying(&view, 80, true);
    assert!(
        unfolded.contains("not shown to this account: the sockets of every other account"),
        "{unfolded}"
    );
}

#[test]
fn a_limited_and_degraded_collector_says_the_failure_first_and_the_limit_under_it() {
    let view = with(
        vigil_model::CollectorState::Degraded,
        Some("the source of logins stopped answering"),
        Some("/private/etc/sudoers is not readable"),
    );

    let page = saying(&view, 80, true);
    let row = page
        .lines()
        .find(|line| line.contains("network"))
        .unwrap_or_else(|| panic!("no row for network: {page}"));
    assert!(row.contains("degraded") && row.contains('!'), "{row}");
    let failure = page
        .find("the source of logins stopped answering")
        .unwrap_or_else(|| panic!("{page}"));
    let limit = page
        .find("not shown to this account: /private/etc/sudoers")
        .unwrap_or_else(|| panic!("{page}"));
    assert!(failure < limit, "{page}");
}
