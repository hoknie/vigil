use vigil_model::Snapshot;
use vigil_view::{Pane, Section, Showing};

use super::super::WhatTheEnginesHold;
use crate::fixture;
use crate::types::Subject;

fn pane(group: &str, name: &str) -> Box<dyn Pane> {
    WhatTheEnginesHold
        .panes()
        .into_iter()
        .find(|pane| pane.belongs_to() == Some(group) && pane.name() == name)
        .unwrap_or_else(|| panic!("no list {name} under {group}"))
}

fn said(pane: &dyn Pane, reading: &Snapshot) -> String {
    let notice = pane
        .why_nothing_is_listed(reading, &Showing::default())
        .expect("an engine list always says why it is empty");
    format!("{} {}", notice.headline, notice.detail.join(" "))
}

#[test]
fn an_engine_not_installed_hides_its_lists_and_says_so_in_one_line() {
    let reading = fixture::only_docker();

    for pane in WhatTheEnginesHold.panes() {
        let podman = pane.belongs_to() == Some("podman");
        assert_eq!(
            pane.shown(&reading),
            !podman,
            "{} of {:?}",
            pane.name(),
            pane.belongs_to()
        );
    }
    assert!(
        said(pane("podman", "containers").as_ref(), &reading)
            .starts_with("podman is not installed on this host."),
    );
}

#[test]
fn an_engine_that_did_not_answer_for_a_list_says_so_and_not_that_it_holds_none() {
    let reading = fixture::docker_silent_on(Subject::Image);
    let images = pane("docker", "images");

    assert!(images.shown(&reading));
    assert!(images.rows(&reading, &Showing::default()).is_empty());
    let why = said(images.as_ref(), &reading);
    assert!(
        why.starts_with("docker did not answer for its images."),
        "{why}"
    );
    assert!(
        why.contains("No finding says any of it was removed"),
        "{why}"
    );
    assert!(
        images
            .tally(&reading, &Showing::default(), 0)
            .contains("docker did not answer for its images"),
    );
}

#[test]
fn an_engine_nobody_named_and_a_dump_nobody_could_read_are_two_other_sentences() {
    let mut unwatched = fixture::engines();
    unwatched.items.retain(|key, _| !key.starts_with("podman|"));
    let mut unread = fixture::engines();
    if let Some(row) = unread.items.get_mut("podman|engine|podman") {
        row["dump_read"] = serde_json::json!(false);
        row["present"] = serde_json::json!(false);
    }
    let containers = pane("podman", "containers");

    assert!(
        said(containers.as_ref(), &unwatched).starts_with("podman is not watched on this host.")
    );
    assert!(
        said(containers.as_ref(), &unread).starts_with("What podman holds is unknown"),
        "a dump that could not be read is not an engine missing from the host, and the two \
         are never turned into one another"
    );
}

#[test]
fn an_engine_that_holds_none_of_a_thing_says_it_answered_and_listed_none() {
    let mut reading = fixture::engines();
    reading
        .items
        .retain(|key, _| !key.starts_with("podman|secret|"));

    assert!(
        said(pane("podman", "secrets").as_ref(), &reading).starts_with("podman holds no secrets.")
    );
}
