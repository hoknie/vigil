use std::collections::BTreeMap;

use serde_json::{Value, json};
use vigil_model::Snapshot;

use super::reading::LaunchReading;
use super::snapshot::{
    AUID_UNSET, CAPPED, DROPPING, LIMIT, SOURCE_ROW, UNNAMED, launches_snapshot,
};
use crate::parsers::launches::audit::Execution;

fn logins() -> BTreeMap<u32, String> {
    BTreeMap::from([(1000, "alice".to_string()), (0, "root".to_string())])
}

fn launch(auid: u32, executable: &str, arguments: &[&str]) -> Execution {
    Execution {
        id: "1757419203.412:3421".into(),
        auid: Some(auid),
        executable: Some(executable.to_string()),
        executable_lossy: false,
        arguments: arguments.iter().map(|word| (*word).to_string()).collect(),
        arguments_lossy: false,
    }
}

fn everything_is_there(_path: &str) -> bool {
    true
}

fn snapshot_of(known: &BTreeMap<String, Value>, executions: &[Execution]) -> Snapshot {
    let logins = logins();
    launches_snapshot(
        "2026-09-09T12:00:00.000Z",
        known,
        &LaunchReading {
            executions,
            logins: &logins,
            any_unnamed: false,
            keep_arguments: false,
            on_disk: &everything_is_there,
            from_plugin: true,
            dropped: false,
        },
    )
}

fn fresh(executions: &[Execution]) -> Snapshot {
    snapshot_of(&BTreeMap::new(), executions)
}

fn people_and_programs(snapshot: &Snapshot) -> Vec<&str> {
    snapshot
        .items
        .keys()
        .filter(|key| key.starts_with("run|"))
        .map(String::as_str)
        .collect()
}

#[test]
fn a_person_and_a_program_are_one_row_under_a_key_somebody_can_copy_into_a_file() {
    let snapshot = fresh(&[launch(1000, "/usr/bin/nc.openbsd", &["nc", "-l"])]);

    assert_eq!(people_and_programs(&snapshot).len(), 1);
    let item = &snapshot.items["run|alice|/usr/bin/nc.openbsd"];
    assert_eq!(item["user"], "alice");
    assert_eq!(item["auid"], 1000);
    assert_eq!(item["first_seen"], "2026-09-09T12:00:00.000Z");
}

#[test]
fn the_same_program_run_a_hundred_times_is_one_row_and_the_row_does_not_move() {
    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);
    let hundred = snapshot_of(
        &once.items,
        &vec![launch(1000, "/usr/bin/nc", &["nc", "-l", "4444"]); 100],
    );

    assert_eq!(once.items, hundred.items);
}

#[test]
fn the_value_of_a_row_that_is_already_there_is_never_rewritten() {
    let logins = logins();
    let gone = |_path: &str| false;
    let known = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]).items;

    let later = launches_snapshot(
        "2026-09-10T09:00:00.000Z",
        &known,
        &LaunchReading {
            executions: &[launch(1000, "/usr/bin/nc", &["nc"])],
            logins: &logins,
            any_unnamed: false,
            keep_arguments: false,
            on_disk: &gone,
            from_plugin: true,
            dropped: false,
        },
    );

    let item = &later.items["run|alice|/usr/bin/nc"];
    assert_eq!(item["exe_present"], json!(true));
    assert_eq!(item["first_seen"], "2026-09-09T12:00:00.000Z");
}

#[test]
fn nothing_a_previous_run_saw_is_ever_dropped() {
    let yesterday = fresh(&[
        launch(1000, "/usr/bin/nc", &["nc"]),
        launch(1000, "/usr/bin/curl", &["curl"]),
    ])
    .items;

    let today = snapshot_of(&yesterday, &[]);

    assert_eq!(today.items, yesterday);
}

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
fn a_loss_becomes_one_row_that_never_moves_afterwards() {
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

    assert_eq!(reading(&first.items, true).items, first.items);
    assert_eq!(reading(&first.items, false).items, first.items);
}

#[test]
fn a_launch_that_belongs_to_no_login_session_is_not_a_person() {
    let snapshot = fresh(&[launch(AUID_UNSET, "/usr/sbin/nginx", &["nginx"])]);

    assert!(people_and_programs(&snapshot).is_empty());
}

#[test]
fn a_login_that_etc_passwd_does_not_name_is_still_a_row_under_its_number() {
    let snapshot = fresh(&[launch(4242, "/opt/app/tool", &["tool"])]);

    assert!(snapshot.items.contains_key("run|4242|/opt/app/tool"));
    assert_eq!(
        snapshot.items["run|4242|/opt/app/tool"]["user"],
        json!(null)
    );
}

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

#[test]
fn a_program_that_is_no_longer_on_disk_says_so_in_the_row_that_recorded_it() {
    let logins = logins();
    let gone = |_path: &str| false;
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
