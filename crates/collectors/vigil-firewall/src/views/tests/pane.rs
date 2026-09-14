use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::{Pane, Room, Section, Showing, conformance};

use super::super::WhatTheHostLetsIn;
use super::super::rows::summary;
use crate::fixture::firewall;
use crate::types::Kind;

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

#[test]
fn a_row_that_left_this_reading_is_still_worth_landing_on() {
    assert!(
        WhatTheHostLetsIn.shows_what_has_gone(),
        "a flushed ruleset is a finding whose object is gone by the time the reader presses \
         o, and that is the whole of what happened: the section opens and says when the row \
         was last seen rather than refusing to move"
    );
}

fn walked(reading: &Snapshot) -> Option<&Value> {
    reading
        .items
        .iter()
        .find(|(key, _)| Kind::of(key) == Some(Kind::Ruleset))
        .map(|(_, item)| item)
}

#[test]
fn the_summary_found_by_its_key_is_the_row_a_walk_over_the_whole_reading_finds() {
    let mut decoyed = firewall();
    for key in [
        "fw-summary-old|nftables",
        "fw-summaryz|nftables",
        "fw-summar|nftables",
    ] {
        decoyed
            .items
            .insert(key.to_string(), json!({"hooked_on_input": 7}));
    }
    let mut without = decoyed.clone();
    without
        .items
        .retain(|key, _| Kind::of(key) != Some(Kind::Ruleset));
    let mut bare = without.clone();
    bare.items
        .insert("fw-summary".to_string(), json!({"hooked_on_input": 2}));

    assert!(walked(&decoyed).is_some() && walked(&bare).is_some());
    for reading in [firewall(), decoyed, without, bare] {
        assert_eq!(
            summary(&reading),
            walked(&reading),
            "a key that only begins like the summary's is not the summary, and a lookup that \
             stops at the first near miss tells the tally the host hooks nothing on input"
        );
    }
}
