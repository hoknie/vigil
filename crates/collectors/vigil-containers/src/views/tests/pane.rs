use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::super::WhatRunsInContainers;
use crate::fixture::containers;

fn pane() -> Box<dyn Pane> {
    WhatRunsInContainers.panes().remove(0)
}

#[test]
fn the_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &containers());
}

#[test]
fn a_container_is_shown_with_what_it_may_do_and_what_of_this_host_it_holds() {
    let reading = containers();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .next()
        .expect("the sample runs a container");

    let said = format!("{:?}", pane.cells(&reading, &row, Room::of(160)));

    assert!(said.contains("yes") || said.contains("no"), "{said}");
}

#[test]
fn a_runtime_socket_is_not_a_row_of_the_table_and_is_still_counted_in_the_footer() {
    let reading = containers();
    let pane = pane();

    let rows = pane.rows(&reading, &Showing::default());
    let footer = pane.tally(&reading, &Showing::default(), rows.len());

    assert!(
        rows.iter().all(|row| row.key.starts_with("container|")),
        "{rows:?}"
    );
    assert!(footer.contains("socket"), "{footer}");
}

#[test]
fn the_detail_of_a_container_holds_every_field_the_agent_wrote_down() {
    let reading = containers();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .next()
        .expect("the sample runs a container");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("runtime"), "{said}");
    assert!(said.contains("host paths"), "{said}");
    assert!(
        said.contains(&row.key),
        "the row says which object it is: {said}"
    );
}
