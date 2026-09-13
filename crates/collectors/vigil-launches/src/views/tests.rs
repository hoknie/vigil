use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::WhatHasRunHere;
use crate::fixture::launches;

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

#[test]
fn the_pane_of_this_module_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &launches());
}

#[test]
fn this_module_puts_its_list_in_the_section_the_processes_also_write_to() {
    assert_eq!(WhatHasRunHere.name(), "programs");
    assert_eq!(pane().name(), "launches");
}

#[test]
fn a_launch_is_shown_under_the_person_who_ran_it() {
    let reading = launches();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("run|"))
        .expect("the sample holds a launch");

    let cells = pane.cells(&reading, &row, Room::of(160));

    assert_eq!(cells.len(), 4);
    assert!(!cells[0].text.is_empty(), "who ran it: {cells:?}");
}

#[test]
fn the_footer_says_that_this_list_only_grows() {
    let reading = launches();
    let pane = pane();

    let footer = pane.tally(&reading, &Showing::default(), 1);

    assert!(
        footer.contains("only grows"),
        "a row that stays after the program is gone is the point of this list: {footer}"
    );
}

#[test]
fn what_the_detail_of_a_launch_says_is_what_a_reader_can_act_on() {
    let reading = launches();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("run|"))
        .expect("the sample holds a launch");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("first seen"), "{said}");
    assert!(
        said.contains(&row.key),
        "the key an operator puts in suppressions is the finding key, and for this family it \
         is the row key whole: {said}"
    );
}
