use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::ui::helpers::words::text;
use crate::ui::screens::summary::render;
use crate::ui::{View, fixture};

fn drawn(width: u16) -> String {
    drawn_from(&fixture::view(), width)
}

fn drawn_from(view: &View, width: u16) -> String {
    let mut buffer = Buffer::empty(Rect::new(0, 0, width, 60));
    render(view, fixture::look(), 0, buffer.area, &mut buffer);
    text::to_text(&buffer)
}

#[test]
fn it_says_where_this_is_what_is_watching_and_where_findings_go() {
    let page = drawn(80);

    assert!(page.contains("app-01"), "{page}");
    assert!(
        page.contains("1c9d8e7b4a5c6d0e"),
        "the published host id: {page}"
    );
    assert!(page.contains("every 30 seconds"), "{page}");
    assert!(page.contains("ports"), "{page}");
    assert!(page.contains("ndjson"), "{page}");
}

#[test]
fn it_says_what_the_agent_is_deliberately_not_saying() {
    let page = drawn(80);

    assert!(page.contains("WHAT IS NOT BEING SAID"), "{page}");
    assert!(
        page.contains("the staging api, expected"),
        "a suppression has to be readable in the operator's own words: {page}"
    );
}

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

fn with_a_collector_switched_off() -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.collectors.push(fixture::collector_off());
    }
    view
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
        .find(|line| line.contains("ports") && line.contains("ok"))
        .expect("a row that is reading");

    assert!(off.contains("off"), "{off}");
    assert!(!off.contains(" ok "), "{off}");
    assert_ne!(
        off.split_whitespace().collect::<Vec<_>>(),
        reading.split_whitespace().collect::<Vec<_>>()
    );
}

#[test]
fn what_the_agent_cannot_do_yet_is_on_the_first_screen() {
    let page = drawn(80);
    let said = fixture::agent::agent()
        .limitations
        .first()
        .cloned()
        .expect("the daemon names what it cannot do yet");

    for word in said.split_whitespace() {
        assert!(
            page.contains(word),
            "a limitation longer than the pane is wrapped rather than cut, and {word} went \
             missing: {page}"
        );
    }
}

#[test]
fn an_agent_that_does_not_report_its_store_says_so_instead_of_drawing_an_empty_one() {
    let page = drawn(80);

    assert!(page.contains("WHAT IS KEPT ON DISK"), "{page}");
    assert!(
        page.contains("does not report what its local history holds"),
        "{page}"
    );
    assert!(
        !page.contains("0 of 0"),
        "an agent that says nothing about its store must not be drawn as an empty store: {page}"
    );
}

fn with_a_store() -> View {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.store = Some(fixture::store());
    }
    view
}

#[test]
fn a_store_that_reports_its_numbers_names_the_file_an_operator_would_run_jq_over() {
    let page = drawn_from(&with_a_store(), 120);

    assert!(page.contains("1284 of 10000"), "{page}");
    assert!(page.contains("2.1 MB"), "{page}");
    assert!(
        page.contains("/var/lib/vigil/findings/journal.ndjson"),
        "the operator needs the place to run jq, and looks for it during an incident: {page}"
    );
    assert!(page.contains("2026-08-27T04:11:53.000Z"), "{page}");
}

#[test]
fn each_of_the_four_numbers_called_dropped_says_which_ceiling_it_is_about() {
    let page = drawn_from(&with_a_store(), 200);

    assert!(page.contains("41 past the retention window"), "{page}");
    assert!(page.contains("0 at the ceiling"), "{page}");
    assert!(
        page.contains("on the findings screen"),
        "the ring in the daemon is a fourth ceiling and has to name itself: {page}"
    );
    assert!(page.contains("nothing is buffered"), "{page}");
}

#[test]
fn the_history_that_does_not_fit_the_screen_is_named_as_kept_and_not_as_dropped() {
    let mut view = with_a_store();
    if let Some(status) = view.status.as_mut() {
        status.agent.findings.retained = 500;
        status.agent.findings.capacity = 500;
        status.agent.findings.total = 1_200;
        status.agent.findings.dropped = 700;
    }

    let page = drawn_from(&view, 200);

    assert!(page.contains("500 of 500 on the findings screen"), "{page}");
    assert!(page.contains("700 of them in the journal only"), "{page}");
    assert!(
        !page.contains("700 dropped"),
        "they are on disk and `jq` reads them: {page}"
    );
    assert!(
        !page.contains("raised"),
        "the word meant this run alone and the number no longer does: {page}"
    );
}

#[test]
fn a_store_with_nothing_in_it_is_named_as_empty_and_not_drawn_as_a_full_one() {
    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.store = Some(vigil_model::StoreStatus {
            records: vigil_model::Counted {
                held: 0,
                ceiling: 10_000,
            },
            journal_path: Some("/var/lib/vigil/findings/journal.ndjson".into()),
            ..vigil_model::StoreStatus::default()
        });
    }

    let page = drawn_from(&view, 120);

    assert!(page.contains("Nothing recorded yet"), "{page}");
    assert!(page.contains("journal.ndjson"), "{page}");
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
        .find(|line| line.contains("ports") && line.contains("ok"))
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
fn the_agent_says_what_it_costs_this_host_or_says_it_was_not_measured() {
    let measured = drawn(80);
    assert!(measured.contains("0.02% of one core"), "{measured}");
    assert!(measured.contains("12 MB resident"), "{measured}");

    let mut view = fixture::view();
    if let Some(status) = view.status.as_mut() {
        status.agent.budget.resident_kb = None;
    }
    let unmeasured = drawn_from(&view, 80);
    assert!(
        unmeasured.contains("does not measure"),
        "not measured here is not measured as zero: {unmeasured}"
    );
}

#[test]
fn a_wide_terminal_puts_the_two_short_lists_side_by_side_instead_of_padding_one() {
    let wide = drawn(160);
    let narrow = drawn(80);

    assert!(wide.contains("THIS HOST, AND THE AGENT"), "{wide}");
    assert!(
        wide.lines()
            .any(|line| line.contains("host") && line.contains("version")),
        "the two lists share a line at this width: {wide}"
    );
    assert!(narrow.contains(" HOST "), "{narrow}");
    assert!(
        !narrow
            .lines()
            .any(|line| line.contains("host") && line.contains("version")),
        "and do not at eighty columns: {narrow}"
    );
}

#[test]
fn a_wide_terminal_also_says_what_each_reading_cost_and_whether_it_has_a_baseline() {
    let wide = drawn(160);

    assert!(wide.contains("BASELINE"), "{wide}");
    assert!(wide.contains("5 ms"), "{wide}");
    assert!(!drawn(80).contains("BASELINE"), "not at eighty, though");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    let off = with_a_collector_switched_off();
    for width in [80u16, 120, 200] {
        for page in [drawn(width), drawn_from(&off, width)] {
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}
