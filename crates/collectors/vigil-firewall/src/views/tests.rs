use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::WhatTheHostLetsIn;
use crate::fixture::firewall;

fn pane() -> Box<dyn Pane> {
    WhatTheHostLetsIn.panes().remove(0)
}

#[test]
fn the_pane_of_this_section_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &firewall());
}

#[test]
fn a_chain_is_shown_with_the_policy_that_decides_what_it_does_with_a_packet() {
    let reading = firewall();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("fw-chain|"))
        .expect("the sample holds a chain");

    let cells = pane.cells(&reading, &row, Room::of(160));
    let said = format!("{cells:?}");

    assert!(said.contains("chain"), "{said}");
    assert!(
        said.contains("drop") || said.contains("accept"),
        "the policy is what a reader came for: {said}"
    );
}

#[test]
fn the_summary_row_comes_first_because_it_is_what_the_whole_screen_is_about() {
    let reading = firewall();

    let rows = pane().rows(&reading, &Showing::default());

    assert!(
        rows.first()
            .is_some_and(|row| row.key.starts_with("fw-summary|")),
        "{rows:?}"
    );
}

#[test]
fn what_the_detail_of_a_chain_says_is_what_a_reader_can_act_on() {
    let reading = firewall();
    let pane = pane();
    let row = pane
        .rows(&reading, &Showing::default())
        .into_iter()
        .find(|row| row.key.starts_with("fw-chain|"))
        .expect("the sample holds a chain");

    let said = format!("{:?}", pane.detail(&reading, &row, 80));

    assert!(said.contains("WHAT THE POLICY MEANS"), "{said}");
    assert!(
        said.contains("firewall|chain|"),
        "the key an operator puts in suppressions is the finding key, with the word the \
         contract uses: {said}"
    );
}
