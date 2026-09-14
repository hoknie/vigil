use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::super::reading::LaunchReading;
use super::super::snapshot::{CAPPED, DROPPING, LIMIT, SOURCE_ROW, UNNAMED, launches_snapshot};

use super::harness::{everything_is_there, launch, logins, snapshot_of};

#[test]
fn every_reading_says_which_of_the_two_sources_it_came_from() {
    let logins = logins();
    let reading = |from_plugin: bool| {
        launches_snapshot(
            "2026-09-09T12:00:00.000Z",
            &BTreeMap::new(),
            &LaunchReading {
                executions: &[],
                logins: &logins,
                any_unnamed: false,
                keep_arguments: false,
                on_disk: &everything_is_there,
                from_plugin,
                dropped: false,
            },
        )
    };

    assert_eq!(reading(true).items[SOURCE_ROW]["from"], "audit plugin");
    assert_eq!(reading(false).items[SOURCE_ROW]["from"], "audit log");
    assert_eq!(reading(true).items, reading(true).items);
}

#[test]
fn a_loss_is_one_row_while_it_lasts_and_leaves_the_reading_when_it_is_over() {
    let logins = logins();
    let reading = |known: &BTreeMap<String, Value>, dropped: bool| {
        launches_snapshot(
            "2026-09-09T12:00:00.000Z",
            known,
            &LaunchReading {
                executions: &[],
                logins: &logins,
                any_unnamed: false,
                keep_arguments: false,
                on_disk: &everything_is_there,
                from_plugin: true,
                dropped,
            },
        )
    };

    let first = reading(&BTreeMap::new(), true);
    assert!(first.items.contains_key(DROPPING));

    assert_eq!(
        reading(&first.items, true).items,
        first.items,
        "a spool still dropping is the same reading and not a second finding"
    );
    assert!(
        !reading(&first.items, false).items.contains_key(DROPPING),
        "everything else in this reading is cumulative, and this row cannot be: a row that \
         never leaves is a finding that can never be closed, and the vocabulary has the \
         closing half of this one"
    );
}

#[test]
fn launches_this_build_could_not_name_are_a_row_rather_than_a_silence() {
    let logins = logins();
    let snapshot = launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        &BTreeMap::new(),
        &LaunchReading {
            executions: &[],
            logins: &logins,
            any_unnamed: true,
            keep_arguments: false,
            on_disk: &everything_is_there,
            from_plugin: true,
            dropped: false,
        },
    );

    assert_eq!(snapshot.items[UNNAMED]["named"], json!(false));
    assert!(
        !snapshot.items.keys().any(|key| key.starts_with("run|")),
        "the marker must not look like a launch"
    );
}

#[test]
fn the_collector_says_when_it_has_stopped_adding_instead_of_growing_for_ever() {
    let mut known: BTreeMap<String, Value> = BTreeMap::new();
    for number in 0..LIMIT {
        known.insert(format!("run|alice|/usr/bin/tool{number}"), json!({}));
    }

    let snapshot = snapshot_of(&known, &[launch(1000, "/usr/bin/one-too-many", &["x"])]);

    assert!(
        !snapshot
            .items
            .contains_key("run|alice|/usr/bin/one-too-many")
    );
    assert_eq!(snapshot.items[CAPPED]["named"], json!(false));
}
