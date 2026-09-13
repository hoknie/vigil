use serde_json::json;
use vigil_module::{Module, Settings};

use super::{Launches, launches::Watching};

fn at_noon() -> vigil_model::Rfc3339 {
    "2026-09-13T12:00:00.000Z".to_string()
}

#[test]
fn a_finding_about_a_launch_walks_to_the_whole_key_because_that_family_adds_no_prefix() {
    assert_eq!(
        Launches.row_of("run|alice|/usr/bin/nc"),
        Some("run|alice|/usr/bin/nc".to_string()),
        "the launches collector keys its own rows this way; cutting the first segment would \
         look for a row that was never written"
    );
}

#[test]
fn the_finding_about_a_dropping_spool_points_at_the_row_that_says_so() {
    assert_eq!(
        Launches.row_of("agent.buffer|launches"),
        Some("launches|dropping".to_string()),
        "the finding is keyed by the buffer and its object is the row about the loss"
    );
    assert_eq!(Launches.row_of("agent.buffer|ndjson"), None);
}

#[test]
fn the_arguments_are_kept_only_when_the_configuration_says_to_keep_them() {
    let quiet: Watching = Settings::plain(at_noon).read().expect("the default");
    let asked: Watching = Settings::of(at_noon, "launches", json!({"record_arguments": true}))
        .read()
        .expect("readable");

    assert!(
        !quiet.record_arguments,
        "a command line holds secrets, so keeping it is a decision an operator makes"
    );
    assert!(asked.record_arguments);
    assert_eq!(Launches.settings_key(), Some("launches"));
}

#[test]
fn a_module_names_the_reading_it_takes_and_how_often_it_takes_it() {
    assert_eq!(Launches.name(), "launches");
    assert_eq!(Launches.every_seconds(), 15);
    assert!(!Launches.rules(&Settings::plain(at_noon)).is_empty());
}
