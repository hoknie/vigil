use std::collections::BTreeMap;

use serde_json::json;

use super::super::reading::LaunchReading;
use super::super::snapshot::launches_snapshot;

use super::harness::{everything_is_there, fresh, launch, logins};

#[test]
fn the_arguments_are_not_in_the_snapshot_unless_the_operator_asked_for_them() {
    let quiet = fresh(&[launch(1000, "/usr/bin/mysql", &["mysql", "-pS3cret"])]);

    assert_eq!(
        quiet.items["run|alice|/usr/bin/mysql"]["arguments"],
        json!(null)
    );
}

#[test]
fn arguments_the_operator_asked_for_arrive_with_their_secrets_already_taken_out() {
    let logins = logins();
    let snapshot = launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        &BTreeMap::new(),
        &LaunchReading {
            executions: &[launch(
                1000,
                "/usr/bin/mysql",
                &["mysql", "-pS3cret", "shop"],
            )],
            logins: &logins,
            any_unnamed: false,
            keep_arguments: true,
            on_disk: &everything_is_there,
            from_plugin: true,
            dropped: false,
        },
    );

    let item = &snapshot.items["run|alice|/usr/bin/mysql"];
    assert_eq!(item["arguments"], "mysql -p[redacted] shop");
    assert_eq!(item["arguments_redacted"], json!(true));
}
