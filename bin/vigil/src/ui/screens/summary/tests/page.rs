use super::harness::{drawn, drawn_from, with_a_collector_switched_off};
use crate::ui::fixture;

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
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    let off = with_a_collector_switched_off();
    let mut losing = fixture::view();
    if let Some(status) = losing.status.as_mut() {
        status.agent.buffers = Some(vec![fixture::losing("ndjson")]);
    }
    for width in [80u16, 120, 200] {
        for page in [
            drawn(width),
            drawn_from(&off, width),
            drawn_from(&losing, width),
        ] {
            for line in page.lines() {
                assert!(
                    line.chars().count() <= width as usize,
                    "{width} columns: {line}"
                );
            }
        }
    }
}

#[test]
fn the_agent_column_starts_in_the_same_place_on_every_line_of_the_summary() {
    let page = drawn(120);
    let block: Vec<&str> = page
        .lines()
        .skip_while(|line| !line.contains("THIS HOST, AND THE AGENT WATCHING IT"))
        .skip(1)
        .take_while(|line| !line.trim().is_empty())
        .collect();

    let at = |label: &str| -> usize {
        block
            .iter()
            .find(|line| line.contains(label))
            .and_then(|line| line.find(label))
            .unwrap_or_else(|| panic!("no line holds {label}: {page}"))
    };

    let first = at("version");
    for label in ["costs", "answered", "findings", "history"] {
        assert_eq!(
            at(label),
            first,
            "{label} sits one column away from the rest of the agent: {page}"
        );
    }
}

#[test]
fn the_field_too_long_for_its_cell_is_cut_a_column_short_of_the_edge() {
    let page = drawn(120);
    let line = page
        .lines()
        .find(|line| line.contains("reads "))
        .unwrap_or_else(|| panic!("no line holds the reading period: {page}"));

    assert!(
        line.contains('…'),
        "this is the field that gets cut: {line}"
    );
    assert!(
        line.chars().count() <= 119,
        "a cut that ends against the edge reads as a broken line: {line}"
    );
}
