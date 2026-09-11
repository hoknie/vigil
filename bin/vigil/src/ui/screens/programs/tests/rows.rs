use super::harness::{drawn, drawn_with, looking_for};
use crate::ui::screens::programs::{keys, rows};
use crate::ui::{Program, Search, fixture};

#[test]
fn a_row_is_the_object_the_collector_recorded_and_its_key_can_be_copied_off_the_screen() {
    let view = fixture::view();

    let named = keys(&view, Program::Running, &Search::default());

    assert!(
        named.contains(&"exec|/usr/sbin/nginx|root".to_string()),
        "{named:?}"
    );
    assert!(
        named.contains(&"processes|unresolved".to_string()),
        "the row about the reading itself is a row with a key, not a footnote: {named:?}"
    );
}

#[test]
fn the_row_that_says_the_reading_stopped_growing_is_not_counted_as_a_program_that_was_run() {
    let view = fixture::view();

    let page = drawn(&view, Program::Running, 80);

    let footer = page
        .lines()
        .rev()
        .find(|line| line.contains("read at"))
        .expect("a footer");
    assert!(
        footer.contains("3 program"),
        "the fixture holds three programs and one row about the reading: {footer}"
    );
    assert!(footer.contains("about the reading itself"), "{footer}");
}

#[test]
fn a_row_the_collector_could_not_read_is_marked_and_sorted_to_the_top_of_its_list() {
    let view = fixture::view();

    let first = rows(&view, Program::Running, &Search::default())
        .into_iter()
        .next()
        .expect("a first row");

    assert!(
        first.mark,
        "a refusal under the fortieth row is a refusal nobody sees"
    );
    assert_eq!(first.key, "processes|unresolved");
}

#[test]
fn a_search_that_matches_nothing_says_it_is_the_search_and_not_the_host() {
    let view = fixture::view();

    let page = drawn_with(&view, Program::Running, &looking_for("postgres"), 80);

    assert!(page.contains("No program matches"), "{page}");
    assert!(page.contains("belongs to this"), "{page}");
}

#[test]
fn a_program_running_from_a_writable_path_is_still_named_by_its_file_and_not_its_path() {
    let page = drawn(&fixture::view(), Program::Running, 80);

    assert!(page.contains("nc (deleted)"), "{page}");
    assert!(
        !page.contains("/tmp/.x/nc"),
        "the path is the detail, not a column of one repeating prefix: {page}"
    );
}
