use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::super::reading::LaunchReading;
use super::super::snapshot::launches_snapshot;
use vigil_collect::Presence;

use super::harness::{fresh, launch, logins};

#[test]
fn a_program_that_is_no_longer_on_disk_says_so_in_the_row_that_recorded_it() {
    let logins = logins();
    let gone = |_path: &str| Presence::Gone;
    let snapshot = launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        &BTreeMap::new(),
        &LaunchReading {
            executions: &[launch(1000, "/tmp/.x/dropper", &["dropper"])],
            logins: &logins,
            any_unnamed: false,
            keep_arguments: false,
            on_disk: &gone,
            from_plugin: true,
            dropped: false,
        },
    );

    let item = &snapshot.items["run|alice|/tmp/.x/dropper"];
    assert_eq!(item["exe_present"], json!(false));
    assert_eq!(item["writable_path"], json!(true));
}

#[test]
fn a_path_the_agent_was_never_shown_is_not_a_file_that_is_gone() {
    let logins = logins();
    let not_shown = |_path: &str| Presence::NotShown;
    let snapshot = launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        &BTreeMap::new(),
        &LaunchReading {
            executions: &[launch(1000, "/tmp/build/tool", &["tool"])],
            logins: &logins,
            any_unnamed: false,
            keep_arguments: false,
            on_disk: &not_shown,
            from_plugin: true,
            dropped: false,
        },
    );

    let item = &snapshot.items["run|alice|/tmp/build/tool"];
    assert_eq!(
        item["exe_present"],
        Value::Null,
        "the shipped unit keeps a /tmp of its own: not seeing a file there is not the file being deleted"
    );
    assert_eq!(item["exe_shown"], json!(false));
    assert_eq!(item["writable_path"], json!(true));
}

#[test]
fn a_file_the_agent_looked_at_says_so_and_the_row_carries_the_answer() {
    let snapshot = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);

    let item = &snapshot.items["run|alice|/usr/bin/nc"];
    assert_eq!(item["exe_present"], json!(true));
    assert_eq!(item["exe_shown"], json!(true));
}
