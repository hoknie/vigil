use serde_json::json;
use vigil_view::{Pane, Piece, Room, Section, Showing, conformance};

use super::super::WhatTheHostLetsIn;
use crate::fixture::firewall;

const AT: usize = 2;

fn pane() -> Box<dyn Pane> {
    WhatTheHostLetsIn.panes().remove(AT)
}

fn row(key: &str) -> vigil_view::RowKey {
    vigil_view::RowKey::of(key)
}

#[test]
fn the_pane_of_the_interfaces_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &firewall());
}

#[test]
fn the_third_pane_of_this_section_is_the_one_that_lists_where_packets_arrive() {
    assert_eq!(
        pane().caption(),
        "WHERE PACKETS ARRIVE",
        "the graph is opened from a row of this pane, so a pane that moved in the section is \
         a key that opens the wrong panel"
    );
    assert!(pane().offers().graph);
}

#[test]
fn an_interface_is_listed_with_the_address_it_answers_on_and_what_went_through_it() {
    let reading = firewall();
    let pane = pane();
    let cells = format!(
        "{:?}",
        pane.cells(&reading, &row("fw-interface|eth0"), Room::of(160))
    );

    assert!(cells.contains("eth0"), "{cells}");
    assert!(cells.contains("192.168.1.23"), "{cells}");
    assert!(
        cells.contains(&(318_841 + 201_773).to_string()),
        "what a reader watches is what went through in both directions: {cells}"
    );
}

#[test]
fn an_interface_nobody_counted_is_listed_with_a_dash_where_the_number_would_be() {
    let mut quiet = firewall();
    quiet.items.insert(
        "fw-interface|eth0".to_string(),
        json!({"name": "eth0", "addresses": [], "the_way_out": false, "counted": false}),
    );

    let cells = format!(
        "{:?}",
        pane().cells(&quiet, &row("fw-interface|eth0"), Room::of(160))
    );

    assert!(cells.contains('—'), "{cells}");
    assert_eq!(
        pane().counted(&quiet, &row("fw-interface|eth0")),
        None,
        "a zero handed to the monitoring panel is a rate of nothing rather than no rate at all"
    );
}

#[test]
fn the_number_the_monitoring_panel_follows_is_the_one_the_reading_holds() {
    let reading = firewall();

    assert_eq!(
        pane().counted(&reading, &row("fw-interface|eth0")),
        Some(318_841 + 201_773)
    );
    assert_eq!(pane().counted(&reading, &row("fw-interface|nothing")), None);
}

#[test]
fn the_graph_of_a_row_draws_the_path_and_the_hooks_of_this_host() {
    let reading = firewall();
    let drawn = pane().graph(&reading, &row("fw-interface|eth0"));
    let said = format!("{drawn:?}");

    assert!(
        said.contains("prerouting") && said.contains("postrouting"),
        "{said}"
    );
    assert!(said.contains("inet filter · input"), "{said}");
    assert!(
        drawn.iter().any(|piece| matches!(piece, Piece::Line(_))),
        "a graph made only of prose is a graph the renderer may rewrap: {said}"
    );
}

#[test]
fn the_footer_says_whether_this_host_is_counting_what_goes_through_it() {
    let reading = firewall();
    let counting = pane().tally(&reading, &Showing::default(), 3);

    assert!(
        counting.contains("counting what goes through"),
        "{counting}"
    );

    let mut quiet = reading.clone();
    for key in [
        "fw-interface|eth0",
        "fw-interface|docker0",
        "fw-interface|lo",
    ] {
        quiet.items.insert(
            key.to_string(),
            json!({"name": "x", "addresses": [], "the_way_out": false, "counted": false}),
        );
    }

    assert!(
        pane()
            .tally(&quiet, &Showing::default(), 3)
            .contains("not counting what goes through"),
        "a reader who opens the graph and finds no number needs the footer to say why"
    );
}
