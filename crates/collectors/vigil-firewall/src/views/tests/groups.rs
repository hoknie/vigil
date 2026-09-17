use serde_json::json;
use vigil_model::Snapshot;
use vigil_view::{Pane, Room, RowKey, Section, Showing, conformance};

use super::super::WhatTheHostLetsIn;
use crate::fixture::firewall;

const AT: usize = 1;

fn pane() -> Box<dyn Pane> {
    WhatTheHostLetsIn.panes().remove(AT)
}

fn behind_firewalld() -> Snapshot {
    let mut reading = firewall();
    reading.items.insert(
        "fw-chain|inet firewalld|filter_INPUT".to_string(),
        json!({
            "family": "inet", "table": "firewalld", "name": "filter_INPUT",
            "type": "filter", "hook": "input", "priority": 10, "policy": "accept",
            "rules": 2,
            "rules_kept": [
                {"handle": 1, "matches": "meta iifname eth0", "does": "jump filter_IN_public"},
                {"handle": 2, "matches": "meta iifname docker0", "does": "jump filter_IN_trusted"}
            ]
        }),
    );
    reading
}

#[test]
fn the_pane_of_the_groups_answers_about_its_own_reading_and_answers_whole() {
    conformance::run_all(pane().as_ref(), &firewall());
    conformance::run_all(pane().as_ref(), &behind_firewalld());
}

#[test]
fn a_table_is_the_group_a_rule_of_this_host_lives_in_and_is_listed_as_one() {
    let reading = firewall();
    let rows = pane().rows(&reading, &Showing::default());

    assert!(
        rows.iter().any(|row| row.key == "fw-table|inet filter"),
        "{rows:?}"
    );
    let cells = format!(
        "{:?}",
        pane().cells(&reading, &RowKey::of("fw-table|inet filter"), Room::of(160))
    );
    assert!(
        cells.contains("table") && cells.contains("inet filter"),
        "{cells}"
    );
}

#[test]
fn a_zone_is_listed_beside_the_tables_although_the_reading_holds_no_row_for_it() {
    let reading = behind_firewalld();
    let rows = pane().rows(&reading, &Showing::default());
    let zone = rows
        .iter()
        .find(|row| row.key == "fw-zone|public")
        .expect("the sample host has a public zone");

    assert!(
        !zone.of_the_reading,
        "a zone is worked out from the jumps on a chain, not sent as a row, and a row the \
         pane made up must say so or a finding will be walked to a key that is not there"
    );
    let cells = format!("{:?}", pane().cells(&reading, zone, Room::of(160)));
    assert!(cells.contains("zone") && cells.contains("eth0"), "{cells}");
}

#[test]
fn the_detail_of_a_zone_says_which_chains_decide_what_happens_in_it() {
    let reading = behind_firewalld();

    let said = format!(
        "{:?}",
        pane().detail(&reading, &RowKey::of("fw-zone|trusted"), 80)
    );

    assert!(said.contains("filter_IN_trusted"), "{said}");
    assert!(said.contains("docker0"), "{said}");
}

#[test]
fn a_host_whose_backend_has_no_zones_says_so_in_the_footer_rather_than_leaving_it_blank() {
    let plain = pane().tally(&firewall(), &Showing::default(), 2);

    assert!(plain.contains("this backend has no zones"), "{plain}");
    assert!(
        pane()
            .tally(&behind_firewalld(), &Showing::default(), 4)
            .contains("2 zone(s)"),
        "a host behind firewalld is counted by the zones firewalld built"
    );
}
