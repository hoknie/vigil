use vigil_model::Snapshot;
use vigil_view::{Facet, Pane, Section, Showing};

use super::super::WhatTheHostLetsIn;
use super::index::scaled;
use super::tallying::tallies_as_the_reading_does;
use crate::fixture::firewall;

fn pane() -> Box<dyn Pane> {
    WhatTheHostLetsIn.panes().remove(0)
}

#[test]
fn the_footer_from_what_was_counted_once_says_what_the_whole_ruleset_says() {
    let pane = pane();
    let nothing = Snapshot::new("firewall", "2026-09-14T09:00:00.000Z".to_string());

    for reading in [firewall(), scaled(4), nothing] {
        tallies_as_the_reading_does(pane.as_ref(), &reading, Showing::default());
    }
}

#[test]
fn the_footer_counts_the_policies_of_the_chains_listed_whatever_else_the_console_keeps() {
    let pane = pane();
    let reading = scaled(4);
    let facets = [Facet::new("kind", "chain")];
    let opened = ["fw-table|inet filter|001"];

    for around in [
        Showing::default().narrowing(&facets),
        Showing::default().hiding(&["chain"]),
        Showing::default().arranged("by table"),
        Showing::default().opening(&opened),
        Showing::default()
            .narrowing(&facets)
            .hiding(&["table"])
            .arranged("by table")
            .opening(&opened),
    ] {
        tallies_as_the_reading_does(pane.as_ref(), &reading, around);
    }
}
