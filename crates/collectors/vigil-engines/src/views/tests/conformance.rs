use vigil_model::Snapshot;
use vigil_view::conformance::{
    every_view_answers_from_its_index_and_its_counts_as_from_the_reading, run_all,
    run_all_of_the_section, the_index_lists_every_search_and_sort_as_the_rows_do_in,
    the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in,
};
use vigil_view::{Facet, Section, Showing};

use super::super::WhatTheEnginesHold;
use crate::fixture;
use crate::types::Subject;

pub(super) fn scaled(copies: usize) -> Snapshot {
    let sample = fixture::engines();
    let mut reading = sample.clone();
    reading.items.clear();
    for (key, item) in &sample.items {
        if key.contains("|engine|") {
            reading.items.insert(key.clone(), item.clone());
            continue;
        }
        for copy in 0..copies {
            reading
                .items
                .insert(format!("{key}-{copy:03}"), item.clone());
        }
    }
    reading
}

fn readings() -> Vec<Snapshot> {
    vec![
        fixture::engines(),
        fixture::only_docker(),
        fixture::docker_silent_on(Subject::Container),
        Snapshot::new("containers-engines", "2026-09-18T09:00:00.000Z".to_string()),
    ]
}

#[test]
fn every_list_of_every_engine_answers_about_its_own_reading_and_answers_whole() {
    for reading in readings() {
        for pane in WhatTheEnginesHold.panes() {
            run_all(pane.as_ref(), &reading);
        }
    }
}

#[test]
fn the_section_gives_every_list_an_engine_and_names_the_engines_in_the_order_they_are_read() {
    run_all_of_the_section(&WhatTheEnginesHold);

    assert_eq!(WhatTheEnginesHold.groups(), vec!["docker", "podman"]);
    assert_eq!(WhatTheEnginesHold.panes().len(), 14);
}

#[test]
fn a_host_of_six_hundred_rows_lists_from_the_index_and_counts_the_footer_as_from_the_reading() {
    let reading = scaled(20);
    assert!(reading.items.len() >= 580, "{}", reading.items.len());

    for pane in WhatTheEnginesHold.panes() {
        every_view_answers_from_its_index_and_its_counts_as_from_the_reading(
            pane.as_ref(),
            &reading,
        );
    }
}

#[test]
fn a_list_narrowed_to_a_compose_project_lists_from_its_index_what_it_lists_from_the_reading() {
    let reading = scaled(3);
    let shop = [Facet::new("project", "shop")];
    let nowhere = [Facet::new("project", "nothing is called this")];
    let foreign = [Facet::new("runtime", "docker")];

    for pane in WhatTheEnginesHold.panes() {
        for only in [&shop[..], &nowhere[..], &foreign[..]] {
            let around = Showing::default().narrowing(only);
            the_index_lists_every_search_and_sort_as_the_rows_do_in(
                pane.as_ref(),
                &reading,
                around,
            );
            the_tally_from_the_counts_says_what_the_tally_from_the_reading_says_in(
                pane.as_ref(),
                &reading,
                around,
            );
        }
    }
}
