use vigil_view::{Pane, Room, Section, Showing, conformance};

use crate::fixture::persistence;
use crate::views::WhatStartsByItself;

fn panes() -> Vec<Box<dyn Pane>> {
    WhatStartsByItself.panes()
}

#[test]
fn every_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    let reading = persistence();

    for pane in panes() {
        if !pane.shown(&reading) {
            continue;
        }
        conformance::run_all(pane.as_ref(), &reading);
    }
}

#[test]
fn a_unit_is_shown_with_the_command_it_runs_and_the_account_it_runs_as() {
    let reading = persistence();
    let units = &panes()[0];
    let row = units
        .rows(&reading, &Showing::default())
        .into_iter()
        .next()
        .expect("the sample holds a unit");

    let said = format!("{:?}", units.cells(&reading, &row, Room::of(160)));

    assert!(said.contains("root") || said.contains("uid"), "{said}");
}

#[test]
fn the_units_are_the_only_list_that_can_be_drawn_as_a_tree() {
    let units = &panes()[0];
    let timers = &panes()[1];

    assert_eq!(
        units
            .arrangements()
            .iter()
            .map(|one| one.name)
            .collect::<Vec<_>>(),
        vec!["list", "tree"],
        "the tree is what one unit file says pulls another in; a timer has no such thing"
    );
    assert!(timers.arrangements().is_empty());
}

#[test]
fn the_tree_puts_a_unit_under_what_pulls_it_in() {
    let reading = persistence();
    let units = &panes()[0];

    let flat = units.rows(&reading, &Showing::default());
    let nested = units.rows(&reading, &Showing::default().arranged("tree"));

    assert!(flat.iter().all(|row| row.depth == 0));
    assert!(
        nested.iter().any(|row| row.depth > 0),
        "nothing was drawn under anything: {nested:?}"
    );
}

#[test]
fn the_row_that_says_the_modules_were_not_readable_is_marked_and_not_counted_as_a_module() {
    let reading = persistence();
    let modules = panes()
        .into_iter()
        .find(|pane| pane.name() == "modules")
        .expect("the modules pane");

    let rows = modules.rows(&reading, &Showing::default());
    let footer = modules.tally(&reading, &Showing::default(), rows.len());

    assert!(
        rows.iter().any(|row| row.key == "modules|unreadable"),
        "{rows:?}"
    );
    assert!(footer.contains("about the reading itself"), "{footer}");
}

#[test]
fn what_the_detail_of_a_unit_says_is_what_a_reader_can_act_on() {
    let reading = persistence();
    let units = &panes()[0];
    let row = units
        .rows(&reading, &Showing::default())
        .into_iter()
        .next()
        .expect("the sample holds a unit");

    let said = format!("{:?}", units.detail(&reading, &row, 80));

    assert!(said.contains("pulled by"), "{said}");
    assert!(
        said.contains(&format!("persistence|{}", row.key)),
        "the key an operator puts in suppressions is the finding key: {said}"
    );
}
