use super::harness::{drawn, drawn_at};
use crate::ui::fixture;

#[test]
fn the_main_screen_fits_eighty_columns_with_every_section_on_it() {
    let page = drawn(&fixture::view(), 80, 30);

    for line in page.lines() {
        assert!(line.chars().count() <= 80, "{line}");
    }
    assert!(page.contains("WHAT IT READS"), "{page}");
    assert!(page.contains("WHAT IT CONCLUDES"), "{page}");
    assert!(page.contains("SECTION"), "{page}");
}

#[test]
fn what_the_agent_reads_comes_before_what_it_makes_of_it() {
    let page = drawn(&fixture::view(), 80, 30);

    let reads = page.find("WHAT IT READS").expect("the first group");
    let concludes = page.find("WHAT IT CONCLUDES").expect("the second group");
    let network = page.find("network").expect("a reading");
    let findings = page.find("findings").expect("a conclusion");

    assert!(
        reads < network && network < concludes && concludes < findings,
        "{page}"
    );
}

#[test]
fn where_the_cursor_is_readable_with_no_colour_at_all() {
    let first = drawn_at(&fixture::view(), 0, 80, 30);
    let second = drawn_at(&fixture::view(), 1, 80, 30);

    assert!(
        first
            .lines()
            .any(|line| line.starts_with(" \u{25b8}   1 network")),
        "{first}"
    );
    assert!(
        second
            .lines()
            .any(|line| line.starts_with(" \u{25b8}   2 accounts")),
        "{second}"
    );
}

#[test]
fn a_wide_terminal_names_the_collector_a_suppression_is_written_against() {
    let wide = drawn(&fixture::view(), 130, 30);
    let narrow = drawn(&fixture::view(), 80, 30);

    assert!(wide.contains("COLLECTOR"), "{wide}");
    assert!(wide.contains("persistence"), "{wide}");
    assert!(!narrow.contains("COLLECTOR"), "{narrow}");
}

#[test]
fn nothing_runs_off_the_side_at_any_of_the_widths_this_is_read_at() {
    for width in [80u16, 120, 200] {
        let page = drawn(&fixture::view(), width, 30);
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width} columns: {line}"
            );
        }
    }
}
