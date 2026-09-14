use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::{Facet, Pane, Section, Showing};

use super::super::WhatTheHostLetsIn;
use super::listing::lists_as_the_rows_do;
use crate::fixture::firewall;

fn pane() -> Box<dyn Pane> {
    WhatTheHostLetsIn.panes().remove(0)
}

pub(super) fn scaled(copies: usize) -> Snapshot {
    let sample = firewall();
    let mut reading = sample.clone();
    reading.items.clear();
    for copy in 0..copies {
        for (key, item) in &sample.items {
            let mut item = item.clone();
            renamed(&mut item, copy);
            reading.items.insert(format!("{key}|{copy:03}"), item);
        }
    }
    reading
}

fn renamed(item: &mut Value, copy: usize) {
    let name = item.get("name").and_then(Value::as_str).map(str::to_string);
    if let Some(name) = name
        && copy % 3 == 1
    {
        item["name"] = json!(format!("{name}-{}", copy % 5));
    }
    let policy = item
        .get("policy")
        .and_then(Value::as_str)
        .map(str::to_string);
    if let Some(policy) = policy
        && copy % 2 == 1
    {
        item["policy"] = json!(match policy.as_str() {
            "drop" => "accept",
            _ => "drop",
        });
    }
}

#[test]
fn the_ruleset_lists_every_search_and_sort_from_its_index_as_its_rows_do() {
    let pane = pane();
    let nothing = Snapshot::new("firewall", "2026-09-14T09:00:00.000Z".to_string());

    assert_eq!(
        lists_as_the_rows_do(pane.as_ref(), &nothing, Showing::default()),
        0
    );
    let sample = firewall();
    assert_eq!(
        lists_as_the_rows_do(pane.as_ref(), &sample, Showing::default()),
        pane.rows(&sample, &Showing::default()).len(),
        "the index holds every row the ruleset lists before anything is typed, and nothing more"
    );
}

#[test]
fn a_larger_ruleset_full_of_ties_lists_from_the_index_in_the_order_it_is_read() {
    let pane = pane();
    let reading = scaled(30);

    assert!(
        lists_as_the_rows_do(pane.as_ref(), &reading, Showing::default()) > 200,
        "thirty copies of one ruleset are what put many chains under one hook and one policy"
    );
}

#[test]
fn nothing_else_the_console_keeps_about_the_list_changes_what_the_ruleset_index_lists() {
    let pane = pane();
    let reading = scaled(3);
    let facets = [Facet::new("kind", "chain")];
    let opened = ["fw-table|inet filter|001"];

    for around in [
        Showing::default().hiding(&["chain"]),
        Showing::default().opening(&opened),
        Showing::default().arranged("by table"),
        Showing::default().narrowing(&facets),
        Showing::default()
            .hiding(&["table"])
            .opening(&opened)
            .arranged("by table")
            .narrowing(&facets)
            .noting(Some("a note")),
    ] {
        lists_as_the_rows_do(pane.as_ref(), &reading, around);
    }
}
