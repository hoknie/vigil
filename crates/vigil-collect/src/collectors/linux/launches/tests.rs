use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use super::LaunchesCollector;
use crate::spool::{Cursor, cursor_path};
use crate::{CollectError, Collector, Health, SpoolWriter};

const LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 items=2 ppid=2143 pid=2170 auid=0 uid=0 tty=pts0 ses=3 comm="id" exe="/usr/bin/id" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0="id""#,
    "\n",
);

const SECOND_LAUNCH: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419204.000:3422): arch=c000003e syscall=59 success=yes exit=0 auid=0 uid=0 comm="uname" exe="/usr/bin/uname" key="vigil_exec""#,
    "\n",
    r#"type=EXECVE msg=audit(1757419204.000:3422): argc=2 a0="uname" a1="-a""#,
    "\n",
);

fn workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("vigil-launches-test");
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join(format!("{}-{name}", std::process::id()));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(spool_beside(&path));
    let _ = fs::remove_file(cursor_path(&spool_beside(&path)));
    let _ = fs::remove_file(plugin_config_beside(&path));
    path
}

fn spool_beside(log: &Path) -> PathBuf {
    PathBuf::from(format!("{}.spool", log.display()))
}

fn plugin_config_beside(log: &Path) -> PathBuf {
    PathBuf::from(format!("{}.plugin-conf", log.display()))
}

fn collector(path: &Path) -> LaunchesCollector {
    LaunchesCollector::with_paths(
        || "2026-09-09T12:00:00.000Z".to_string(),
        false,
        spool_beside(path),
        path,
        plugin_config_beside(path),
    )
}

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
fn a_host_with_no_auditd_says_so_instead_of_reporting_that_nobody_ran_anything() {
    let path = workspace("absent.log");
    let collector = collector(&path);

    assert!(matches!(collector.available(), Health::Unavailable(_)));
    assert!(matches!(collector.collect(), Err(CollectError::Absent(_)),));
}

#[test]
fn a_log_without_our_tag_in_it_is_a_degraded_collector_and_not_a_healthy_one() {
    let path = workspace("untagged.log");
    fs::write(
        &path,
        "type=USER_LOGIN msg=audit(1757419203.412:3400): pid=1 uid=0 auid=1000 res=success\n",
    )
    .expect("write");

    match collector(&path).available() {
        Health::Degraded(detail) => assert!(detail.contains("augenrules"), "{detail}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_spool_is_read_in_preference_to_the_log_and_the_reading_says_which_it_was() {
    let path = workspace("preferred.log");
    fs::write(&path, LAUNCH).expect("write");
    let mut spool = SpoolWriter::open(spool_beside(&path), 1024 * 1024).expect("opens");
    spool.write(SECOND_LAUNCH.as_bytes()).expect("writes");

    let snapshot = collector(&path).collect().expect("readable");

    assert!(
        snapshot.items.contains_key("run|root|/usr/bin/uname"),
        "the spool is the source when it holds anything"
    );
    assert!(
        !snapshot.items.contains_key("run|root|/usr/bin/id"),
        "and the log is not read as well — that would be two sources for one host"
    );
    assert_eq!(snapshot.items["launches|source"]["from"], "audit plugin");
}

#[test]
fn an_empty_spool_does_not_stop_the_log_being_read() {
    let path = workspace("empty-spool.log");
    fs::write(&path, LAUNCH).expect("write");
    SpoolWriter::open(spool_beside(&path), 1024 * 1024).expect("opens");

    let snapshot = collector(&path).collect().expect("readable");

    assert!(snapshot.items.contains_key("run|root|/usr/bin/id"));
    assert_eq!(snapshot.items["launches|source"]["from"], "audit log");
}

#[test]
fn a_host_that_gains_a_plugin_while_the_daemon_runs_moves_onto_it() {
    let path = workspace("switched.log");
    fs::write(&path, LAUNCH).expect("write");
    let collector = collector(&path);
    let before = collector.collect().expect("readable");
    assert_eq!(before.items["launches|source"]["from"], "audit log");

    let mut spool = SpoolWriter::open(spool_beside(&path), 1024 * 1024).expect("opens");
    spool.write(SECOND_LAUNCH.as_bytes()).expect("writes");
    let after = collector.collect().expect("readable");

    assert_eq!(after.items["launches|source"]["from"], "audit plugin");
    assert!(
        after.items.contains_key("run|root|/usr/bin/id"),
        "what the log had already said is not un-said by the change of source"
    );
    assert!(after.items.contains_key("run|root|/usr/bin/uname"));
}

#[test]
fn the_reader_leaves_a_cursor_the_plugin_can_reclaim_space_with() {
    let path = workspace("cursor.log");
    let spool_path = spool_beside(&path);
    let mut spool = SpoolWriter::open(spool_path.clone(), 1024 * 1024).expect("opens");
    spool.write(LAUNCH.as_bytes()).expect("writes");

    collector(&path).collect().expect("readable");

    let cursor = Cursor::read(&cursor_path(&spool_path)).expect("a cursor was written");
    assert_eq!(
        cursor.offset,
        LAUNCH.len() as u64,
        "the cursor must name the end of what was read"
    );
    assert_eq!(cursor.inode, fs::metadata(&spool_path).expect("stat").ino());
}

#[test]
fn a_spool_that_had_to_drop_events_says_so_in_a_row_and_in_its_health() {
    let path = workspace("dropping.log");
    let spool_path = spool_beside(&path);
    let mut spool = SpoolWriter::open(spool_path.clone(), 4 * 1024).expect("opens");
    for _ in 0..100 {
        spool.write(LAUNCH.as_bytes()).expect("writes");
    }
    assert!(spool.report().dropped > 0, "the ceiling must have bitten");

    let collector = collector(&path);
    let snapshot = collector.collect().expect("readable");

    assert!(
        snapshot.items.contains_key("launches|dropping"),
        "{:?}",
        snapshot.items.keys().collect::<Vec<_>>()
    );
    match collector.available() {
        Health::Degraded(detail) => assert!(detail.contains("dropped"), "{detail}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_registered_plugin_that_brings_nothing_is_not_the_same_answer_as_a_missing_rule() {
    let path = workspace("silent-plugin.log");
    fs::write(&path, LAUNCH).expect("write");
    fs::write(
            plugin_config_beside(&path),
            "active = yes\ndirection = out\npath = /usr/sbin/vigil-audit-plugin\ntype = always\nformat = string\n",
        )
        .expect("write");

    match collector(&path).available() {
        Health::Degraded(detail) => {
            assert!(detail.contains("has never run"), "{detail}");
            assert!(detail.contains("restart auditd"), "{detail}");
            assert!(!detail.contains("augenrules"), "{detail}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_plugin_that_has_started_and_delivered_nothing_is_not_told_to_restart_anything() {
    let path = workspace("quiet-plugin.log");
    fs::write(&path, LAUNCH).expect("write");
    fs::write(plugin_config_beside(&path), "active = yes\n").expect("write");
    SpoolWriter::open(spool_beside(&path), 1024 * 1024).expect("opens");

    match collector(&path).available() {
        Health::Degraded(detail) => {
            assert!(detail.contains("delivered nothing"), "{detail}");
            assert!(!detail.contains("has never run"), "{detail}");
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_host_reading_the_log_with_no_plugin_registered_is_healthy() {
    let path = workspace("fallback.log");
    fs::write(&path, LAUNCH).expect("write");

    assert_eq!(collector(&path).available(), Health::Ok);
}

#[test]
fn the_agent_never_says_a_file_it_was_not_shown_has_been_deleted() {
    use super::reading::look_on_disk;
    use crate::Presence;

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
