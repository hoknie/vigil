use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::conformance::{
    every_view_answers_from_its_index_and_its_counts_as_from_the_reading,
    the_index_lists_every_search_and_sort_as_the_rows_do_in,
};
use vigil_view::{Facet, Pane, Section, Showing};

use super::super::WhatRunsInContainers;
use crate::fixture::containers;

fn pane() -> Box<dyn Pane> {
    WhatRunsInContainers.panes().remove(0)
}

pub(super) fn scaled(copies: usize) -> Snapshot {
    let sample = containers();
    let mut reading = sample.clone();
    reading.items.clear();
    for copy in 0..copies {
        for (key, item) in &sample.items {
            let mut item = item.clone();
            moved(&mut item, copy);
            reading.items.insert(format!("{key}{copy:03}"), item);
        }
    }
    reading
}

fn moved(item: &mut Value, copy: usize) {
    let path = item.get("exe").and_then(Value::as_str).map(str::to_string);
    if let Some(path) = path
        && copy % 3 == 1
    {
        item["exe"] = json!(format!("/opt/release-{}{path}", copy % 5));
    }
    if item.get("runtime").is_some() && copy % 4 == 2 {
        item["runtime"] = json!("containerd");
    }
}

fn indexed(pane: &dyn Pane, reading: &Snapshot) -> usize {
    pane.index(reading, &Showing::default())
        .expect("the containers answer from an index")
        .len()
}

#[test]
fn the_containers_list_every_search_and_sort_from_their_index_as_their_rows_do() {
    let pane = pane();
    let nothing = Snapshot::new("containers", "2026-09-14T09:00:00.000Z".to_string());
    let sample = containers();

    for reading in [&nothing, &sample] {
        the_index_lists_every_search_and_sort_as_the_rows_do_in(
            pane.as_ref(),
            reading,
            Showing::default(),
        );
    }
    assert_eq!(
        indexed(pane.as_ref(), &nothing),
        0,
        "a host read with no containers indexes nothing, or a search would find containers that \
         are not there"
    );
    assert_eq!(
        indexed(pane.as_ref(), &sample),
        pane.rows(&sample, &Showing::default()).len(),
        "the index holds the containers and not the runtime sockets, which the footer counts \
         and the table does not list"
    );
}

#[test]
fn a_larger_host_full_of_ties_lists_its_containers_from_the_index_in_the_order_they_are_read() {
    let pane = pane();
    let reading = scaled(60);

    every_view_answers_from_its_index_and_its_counts_as_from_the_reading(pane.as_ref(), &reading);
    assert!(
        indexed(pane.as_ref(), &reading) >= 120,
        "sixty copies of two containers are what put many rows under one runtime and one answer \
         about the host"
    );
}

#[test]
fn nothing_else_the_console_keeps_about_the_list_changes_what_the_containers_index_lists() {
    let pane = pane();
    let reading = scaled(4);
    let facets = [Facet::new("runtime", "docker")];
    let opened = ["container|3ab1c0f2d4e5001"];

    for around in [
        Showing::default().hiding(&["socket"]),
        Showing::default().opening(&opened),
        Showing::default().arranged("by runtime"),
        Showing::default().narrowing(&facets),
        Showing::default()
            .hiding(&["container"])
            .opening(&opened)
            .arranged("by runtime")
            .narrowing(&facets)
            .noting(Some("a note")),
    ] {
        the_index_lists_every_search_and_sort_as_the_rows_do_in(pane.as_ref(), &reading, around);
    }
}
