use vigil_model::Snapshot;
use vigil_view::conformance::the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in;
use vigil_view::{Facet, Pane, Section, Showing};

use super::super::WhatRunsInContainers;
use super::index::scaled;
use crate::fixture::containers;

fn pane() -> Box<dyn Pane> {
    WhatRunsInContainers.panes().remove(0)
}

#[test]
fn the_footer_from_what_was_counted_once_says_what_the_whole_host_says() {
    let pane = pane();
    let nothing = Snapshot::new("containers", "2026-09-14T09:00:00.000Z".to_string());

    for reading in [containers(), scaled(6), nothing] {
        the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
            pane.as_ref(),
            &reading,
            Showing::default(),
        );
    }
}

#[test]
fn the_footer_counts_the_same_containers_and_sockets_whatever_else_the_console_keeps() {
    let pane = pane();
    let reading = scaled(6);
    let facets = [Facet::new("runtime", "docker")];
    let opened = ["container|3ab1c0f2d4e5001"];

    for around in [
        Showing::default().narrowing(&facets),
        Showing::default().hiding(&["socket"]),
        Showing::default().arranged("by runtime"),
        Showing::default().opening(&opened),
        Showing::default()
            .narrowing(&facets)
            .hiding(&["container"])
            .arranged("by runtime")
            .opening(&opened),
    ] {
        the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
            pane.as_ref(),
            &reading,
            around,
        );
    }
}
