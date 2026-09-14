use std::fs;
use std::os::unix::fs::MetadataExt;

use crate::SpoolWriter;
use crate::spool::{Cursor, cursor_path};
use vigil_collect::{Collector, Health};

use super::harness::{LAUNCH, SECOND_LAUNCH, collector, spool_beside, workspace};

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
