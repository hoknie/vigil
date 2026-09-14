use vigil_view::{Pane, Room, Section, Showing, conformance};

use crate::fixture::processes;
use crate::views::WhatHasRunHere;

fn pane() -> Box<dyn Pane> {
    WhatHasRunHere.panes().remove(0)
}

#[test]
fn the_pane_of_this_module_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &processes());
}

#[test]
fn this_module_puts_its_list_in_the_section_the_launches_also_write_to() {
    assert_eq!(WhatHasRunHere.name(), "programs");
    assert_eq!(pane().name(), "running");
}

#[test]
fn a_program_is_shown_with_the_account_it_runs_as_and_what_started_it() {
    let reading = processes();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| !row.key.starts_with("processes|"))
        .expect("the sample runs a program");

    let cells = pane.cells(&reading, &row, Room::of(160));

    assert_eq!(cells.len(), 4);
    assert!(!cells[1].text.is_empty(), "{cells:?}");
}

#[test]
fn the_row_about_the_reading_itself_is_listed_first_and_counted_apart() {
    let reading = processes();
    let pane = pane();

    let rows = pane.rows(&reading, &Showing::default());
    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(
        rows.first()
            .is_some_and(|row| row.key.starts_with("processes|")),
        "what the reading could not finish is the first thing a reader sees, not the last: \
         {rows:?}"
    );
    assert!(footer.contains("about the reading itself"), "{footer}");
}

#[test]
fn what_the_detail_of_a_program_says_is_what_a_reader_can_act_on() {
    let reading = processes();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| !row.key.starts_with("processes|"))
        .expect("the sample runs a program");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("started by"), "{said}");
    assert!(
        said.contains(&format!("process|{}", row.key)),
        "the key an operator puts in suppressions is the finding key: {said}"
    );
}
