use serde_json::{Value, json};

use super::super::reading::LaunchReading;
use super::super::snapshot::launches_snapshot;
use crate::parsers::audit::Execution;
use vigil_collect::Presence;

use super::harness::{fresh, later, launch, logins, people_and_programs, snapshot_of};

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
fn the_same_program_run_a_hundred_times_is_one_row_that_counts_them_and_moves_nothing_else() {
    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);
    let runs: Vec<Execution> = (0..100)
        .map(|serial| later(1000, "/usr/bin/nc", &["nc", "-l", "4444"], serial))
        .collect();
    let hundred = snapshot_of(&once.items, &runs);

    assert_eq!(people_and_programs(&hundred).len(), 1);
    assert_eq!(once.items["run|alice|/usr/bin/nc"]["runs"], json!(1));
    assert_eq!(hundred.items["run|alice|/usr/bin/nc"]["runs"], json!(101));
    assert_eq!(
        hundred.items["run|alice|/usr/bin/nc"]["last_audit_id"],
        json!("1757419300.000:4099")
    );

    let mut uncounted = hundred.items["run|alice|/usr/bin/nc"].clone();
    uncounted["runs"] = json!(1);
    uncounted["last_audit_id"] = once.items["run|alice|/usr/bin/nc"]["last_audit_id"].clone();
    assert_eq!(
        uncounted, once.items["run|alice|/usr/bin/nc"],
        "the count is the one field a later run writes: the arguments, the audit id and the \
         moment it was first seen still belong to the first run, which is the one a reader \
         looks up in the host's own log"
    );
}

#[test]
fn a_row_an_older_agent_wrote_without_a_count_is_counted_from_the_run_it_recorded() {
    let mut known = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]).items;
    if let Some(fields) = known
        .get_mut("run|alice|/usr/bin/nc")
        .and_then(Value::as_object_mut)
    {
        fields.remove("runs");
        fields.remove("last_audit_id");
    }

    let again = snapshot_of(&known, &[later(1000, "/usr/bin/nc", &["nc"], 1)]);
    assert_eq!(again.items["run|alice|/usr/bin/nc"]["runs"], json!(2));

    let the_first_run_again = snapshot_of(&known, &[launch(1000, "/usr/bin/nc", &["nc"])]);
    assert_eq!(
        the_first_run_again.items["run|alice|/usr/bin/nc"]["runs"],
        Value::Null,
        "without a last counted id the row's own audit id stands in for it, so the run it was \
         written from is not counted a second time"
    );
}

#[test]
fn records_read_a_second_time_after_a_restart_are_not_counted_a_second_time() {
    let once = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]);
    let tail = [
        launch(1000, "/usr/bin/nc", &["nc"]),
        later(1000, "/usr/bin/nc", &["nc"], 1),
        later(1000, "/usr/bin/nc", &["nc"], 2),
    ];

    let read = snapshot_of(&once.items, &tail);
    let read_again = snapshot_of(&read.items, &tail);

    assert_eq!(read.items["run|alice|/usr/bin/nc"]["runs"], json!(3));
    assert_eq!(
        read_again.items, read.items,
        "an agent restarted without a cursor reads the tail of the audit log again, and a \
         count that grows with every restart counts restarts, not runs"
    );
}

#[test]
fn a_run_that_arrives_later_is_counted_even_when_its_serial_is_smaller_after_a_reboot() {
    let once = fresh(&[later(1000, "/usr/bin/nc", &["nc"], 90)]);
    let after_a_reboot = Execution {
        id: "1757500000.000:12".into(),
        ..launch(1000, "/usr/bin/nc", &["nc"])
    };

    let read = snapshot_of(&once.items, &[after_a_reboot]);

    assert_eq!(read.items["run|alice|/usr/bin/nc"]["runs"], json!(2));
}

#[test]
fn a_count_that_moves_raises_no_finding_because_the_rules_answer_only_a_new_row() {
    let once = fresh(&[launch(1000, "/tmp/.x/dropper", &["dropper"])]);
    let again = snapshot_of(
        &once.items,
        &[later(1000, "/tmp/.x/dropper", &["dropper"], 1)],
    );

    let changes = vigil_rules::diff(&once, &again);

    assert_eq!(changes.len(), 1, "{changes:?}");
    assert!(
        vigil_rules::findings_for(crate::rules::launch_rules(), &changes).is_empty(),
        "a dropper run from /tmp raises its finding the first time; the hundredth run of it \
         is a counter on the row, not a hundred findings"
    );
}

#[test]
fn a_row_that_is_already_there_keeps_what_it_first_recorded_and_only_its_count_moves() {
    let logins = logins();
    let gone = |_path: &str| Presence::Gone;
    let known = fresh(&[launch(1000, "/usr/bin/nc", &["nc"])]).items;

    let later = launches_snapshot(
        "2026-09-10T09:00:00.000Z",
        &known,
        &LaunchReading {
            executions: &[later(1000, "/usr/bin/nc", &["nc"], 1)],
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
    assert_eq!(item["runs"], json!(2));
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
