use vigil_view::{Room, Section, Showing};

use super::{Unknown, unknown_readings};
use crate::ui::fixture;

fn with_a_reading_nobody_draws() -> crate::ui::View {
    let mut view = fixture::view();
    let (status, reading) = fixture::of_a_reading_with_no_screen();
    view.status = Some(status);
    view.readings.put("kernel", reading);
    view
}

#[test]
fn a_reading_no_section_of_this_build_draws_is_a_list_of_its_own() {
    let view = with_a_reading_nobody_draws();

    assert_eq!(unknown_readings(&view), vec!["kernel".to_string()]);
    let section = Unknown::of(&view);
    let panes = section.panes();

    assert_eq!(panes.len(), 1);
    assert_eq!(panes[0].reads(), "kernel");
    assert_eq!(
        panes[0].name(),
        "kernel",
        "the row of names says which reading it is, because nothing else on the screen can"
    );
}

#[test]
fn every_row_the_agent_sent_is_drawn_with_the_values_it_recorded() {
    let view = with_a_reading_nobody_draws();
    let section = Unknown::of(&view);
    let pane = section.panes().remove(0);
    let crate::ui::Reading::Taken(reading) = view.reading("kernel") else {
        panic!("the sample carries the reading");
    };

    let rows = pane.rows(reading, &Showing::default());
    assert!(!rows.is_empty(), "{rows:?}");

    let drawn = format!("{:?}", pane.cells(reading, &rows[0], Room::of(160)));
    assert!(drawn.contains("module"), "the class of the row: {drawn}");
    assert!(
        drawn.contains("="),
        "and the values beside it, named: {drawn}"
    );
}

#[test]
fn the_detail_of_a_row_holds_every_value_the_agent_recorded_about_it() {
    let view = with_a_reading_nobody_draws();
    let section = Unknown::of(&view);
    let pane = section.panes().remove(0);
    let crate::ui::Reading::Taken(reading) = view.reading("kernel") else {
        panic!("the sample carries the reading");
    };
    let row = pane
        .rows(reading, &Showing::default())
        .into_iter()
        .next()
        .expect("a row");

    let said = format!("{:?}", pane.detail(reading, &row, 80));

    assert!(said.contains("tainted"), "{said}");
    assert!(
        said.contains(&row.key),
        "the row says which object it is: {said}"
    );
}

#[test]
fn a_reading_nobody_draws_is_searched_and_sorted_from_its_index_as_from_the_reading() {
    let view = with_a_reading_nobody_draws();
    let section = Unknown::of(&view);
    let pane = section.panes().remove(0);
    let crate::ui::Reading::Taken(reading) = view.reading("kernel") else {
        panic!("the sample carries the reading");
    };

    assert!(
        pane.index(reading, &Showing::default()).is_some(),
        "the console answers every keystroke on this list from an index, so the list has to \
         give it one"
    );
    vigil_view::conformance::the_index_lists_every_search_and_sort_as_the_rows_do(
        pane.as_ref(),
        reading,
    );
    vigil_view::conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says(
        pane.as_ref(),
        reading,
    );
}

#[test]
fn a_reading_this_build_has_a_section_for_is_never_listed_here() {
    let view = fixture::view();
    let listed = unknown_readings(&view);

    for drawn in ["ports", "users", "firewall", "containers", "files"] {
        assert!(
            !listed.contains(&drawn.to_string()),
            "{drawn} has a screen of its own and is listed here as well, which is the same \
             rows in two places"
        );
    }
    assert_eq!(
        listed,
        Vec::<String>::new(),
        "what a screen of this build draws and what only this list draws is one fact, and it \
         is stated here so that a reading quietly losing its screen is caught: every reading of \
         this build has a screen, the engines' among them"
    );
    assert_eq!(Unknown::of(&view).panes().len(), listed.len());
}
