use std::fs;

use vigil_collect::Collector;

use super::harness::{LAUNCH, SECOND_LAUNCH, collector, workspace};

#[test]
fn a_launch_in_the_log_becomes_a_row_under_the_person_who_ran_it() {
    let path = workspace("one.log");
    fs::write(&path, LAUNCH).expect("write");

    let snapshot = collector(&path).collect().expect("readable");

    assert_eq!(snapshot.source, "launches");
    assert!(
        snapshot.items.contains_key("run|root|/usr/bin/id"),
        "{:?}",
        snapshot.items.keys().collect::<Vec<_>>()
    );
}

#[test]
fn what_was_read_once_is_not_read_again_and_what_was_seen_once_is_not_lost() {
    let path = workspace("appended.log");
    fs::write(&path, LAUNCH).expect("write");
    let collector = collector(&path);

    let first = collector.collect().expect("readable");
    fs::write(&path, format!("{LAUNCH}{SECOND_LAUNCH}")).expect("append");
    let second = collector.collect().expect("readable");

    assert!(first.items.contains_key("run|root|/usr/bin/id"));
    assert!(second.items.contains_key("run|root|/usr/bin/id"));
    assert!(second.items.contains_key("run|root|/usr/bin/uname"));
    assert_eq!(second.items.len(), first.items.len() + 1);
}

#[test]
fn a_reading_that_finds_nothing_new_is_exactly_the_previous_reading() {
    let path = workspace("still.log");
    fs::write(&path, LAUNCH).expect("write");
    let collector = collector(&path);

    let first = collector.collect().expect("readable");
    let second = collector.collect().expect("readable");

    assert_eq!(first.items, second.items, "an idle host must be silent");
}

#[test]
fn a_rotated_log_is_read_from_its_beginning_rather_than_from_an_offset_into_nothing() {
    let path = workspace("rotated.log");
    fs::write(&path, format!("{LAUNCH}{SECOND_LAUNCH}")).expect("write");
    let collector = collector(&path);
    collector.collect().expect("readable");

    fs::remove_file(&path).expect("rotate");
    fs::write(&path, LAUNCH).expect("fresh log");
    let after = collector.collect().expect("readable");

    assert!(after.items.contains_key("run|root|/usr/bin/id"));
    assert!(
        after.items.contains_key("run|root|/usr/bin/uname"),
        "rotation must not lose what was already known"
    );
}

#[test]
fn a_daemon_that_restarts_carries_on_from_what_it_knew() {
    let path = workspace("restart.log");
    fs::write(&path, LAUNCH).expect("write");
    let before = collector(&path).collect().expect("readable");

    let after_restart = collector(&path);
    after_restart.restore(&before);
    fs::write(&path, SECOND_LAUNCH).expect("write");
    let first_tick = after_restart.collect().expect("readable");

    assert!(
        first_tick.items.contains_key("run|root|/usr/bin/id"),
        "what the previous run knew must survive the restart"
    );
    assert!(first_tick.items.contains_key("run|root|/usr/bin/uname"));
}

#[test]
fn the_agent_never_says_a_file_it_was_not_shown_has_been_deleted() {
    use super::super::reading::look_on_disk;
    use vigil_collect::Presence;

    assert_eq!(
        look_on_disk("/tmp/there-is-no-such-file"),
        Presence::NotShown
    );
    assert_eq!(
        look_on_disk("/var/tmp/there-is-no-such-file"),
        Presence::NotShown
    );
    assert_eq!(
        look_on_disk("/usr/there-is-no-such-file"),
        Presence::Gone,
        "outside the two private directories the agent does see the host"
    );
    assert_eq!(look_on_disk("/etc/passwd"), Presence::OnDisk);
}
