use super::SpoolWriter;
use super::on_disk::inode_of;
use super::writing::MAX_RECORD_BYTES;
use crate::spool::cursor::Cursor;
use crate::spool::marker;
use crate::spool::place::cursor_path;
use std::fs;
use std::path::{Path, PathBuf};

const SYSCALL: &str = concat!(
    r#"type=SYSCALL msg=audit(1757419203.412:3421): arch=c000003e syscall=59 success=yes exit=0 auid=1000 uid=1000 comm="id" exe="/usr/bin/id" key="vigil_exec""#,
    "\n"
);
const EXECVE: &str = "type=EXECVE msg=audit(1757419203.412:3421): argc=1 a0=\"id\"\n";
const PROCTITLE: &str =
    "type=PROCTITLE msg=audit(1757419203.412:3421): proctitle=2F7573722F62696E2F6964\n";
const LOGIN: &str =
    "type=USER_LOGIN msg=audit(1757419203.400:3400): pid=1 uid=0 auid=1000 res=success\n";

fn workspace(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join("vigil-spool-test");
    fs::create_dir_all(&directory).expect("temp dir");
    let path = directory.join(format!("{}-{name}", std::process::id()));
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(cursor_path(&path));
    path
}

fn contents(path: &Path) -> String {
    fs::read_to_string(path).expect("readable")
}

#[test]
fn what_the_collector_reads_is_written_and_what_it_never_reads_is_not() {
    let path = workspace("filter");
    let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

    spool
        .write(format!("{LOGIN}{SYSCALL}{PROCTITLE}{EXECVE}").as_bytes())
        .expect("writes");

    let written = contents(&path);
    assert!(written.contains("type=SYSCALL"), "{written}");
    assert!(written.contains("type=EXECVE"), "{written}");
    assert!(!written.contains("USER_LOGIN"), "{written}");
    assert!(!written.contains("PROCTITLE"), "{written}");
    assert_eq!(spool.report().kept, 2);
    assert_eq!(spool.report().skipped, 2);
}

#[test]
fn a_record_split_across_two_reads_is_written_once_and_whole() {
    let path = workspace("split");
    let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");
    let (head, tail) = SYSCALL.split_at(40);

    spool.write(head.as_bytes()).expect("writes");
    assert_eq!(contents(&path), "", "half a record is not a record");
    spool.write(tail.as_bytes()).expect("writes");

    assert_eq!(contents(&path), SYSCALL);
}

#[test]
fn a_line_longer_than_any_record_is_given_up_on_rather_than_buffered_for_ever() {
    let path = workspace("oversized");
    let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

    spool
        .write(&vec![b'x'; MAX_RECORD_BYTES + 10])
        .expect("writes");
    spool
        .write(format!("xxxxx\n{SYSCALL}").as_bytes())
        .expect("writes");

    assert_eq!(spool.report().oversized, 1);
    assert_eq!(
        contents(&path),
        SYSCALL,
        "the record after the damage must survive"
    );
}

#[test]
fn the_ceiling_drops_the_oldest_and_says_so_when_nothing_is_reading() {
    let path = workspace("ceiling");
    let ceiling = 8 * 1024;
    let mut spool = SpoolWriter::open(path.clone(), ceiling).expect("opens");

    for _ in 0..200 {
        spool.write(SYSCALL.as_bytes()).expect("writes");
    }

    assert!(
        spool.bytes() <= ceiling,
        "the spool grew past its ceiling: {} bytes",
        spool.bytes()
    );
    assert!(spool.report().dropped > 0, "a loss must be counted");
    let written = contents(&path);
    let first = written.lines().next().expect("a first line");
    assert!(
        marker::dropped_note(first.as_bytes()).is_some(),
        "the loss must be announced at the head of the file: {first}"
    );
    for line in written.lines().skip(1) {
        assert!(line.starts_with("type="), "cut mid-record: {line}");
    }
}

#[test]
fn what_the_reader_has_already_consumed_is_reclaimed_without_anything_being_lost() {
    let path = workspace("consumed");
    let ceiling = 8 * 1024;
    let mut spool = SpoolWriter::open(path.clone(), ceiling).expect("opens");
    for _ in 0..30 {
        spool.write(SYSCALL.as_bytes()).expect("writes");
    }
    assert!(spool.bytes() < ceiling, "the first batch must fit");

    let inode = inode_of(&spool.file).expect("stat");
    Cursor {
        inode,
        offset: spool.bytes(),
    }
    .write(&cursor_path(&path))
    .expect("cursor");

    for _ in 0..30 {
        spool.write(SYSCALL.as_bytes()).expect("writes");
    }

    assert_eq!(spool.report().compactions, 1, "it should have compacted");
    assert_eq!(spool.report().dropped, 0, "nothing was lost");
    assert!(
        !cursor_path(&path).exists(),
        "the cursor of the file that has just been replaced must not survive it — inode \
         numbers are reused, and a stale cursor that passes the inode test is a silent loss"
    );
    let written = contents(&path);
    assert!(
        marker::dropped_note(written.lines().next().unwrap_or("").as_bytes()).is_none(),
        "a compaction that lost nothing must not claim it did: {written}"
    );
    assert_eq!(written.lines().count(), 30, "only what was unread is left");
}

#[test]
fn a_spool_torn_by_a_power_cut_costs_one_record_and_not_the_one_behind_it() {
    let path = workspace("torn");
    fs::write(&path, format!("{SYSCALL}type=SYSCALL msg=aud")).expect("writes");

    let mut spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");
    spool.write(EXECVE.as_bytes()).expect("writes");

    let written = contents(&path);
    let lines: Vec<&str> = written.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(
        lines[1], "type=SYSCALL msg=aud",
        "the torn record stays torn"
    );
    assert_eq!(
        lines[2],
        EXECVE.trim_end(),
        "and the record after it is whole"
    );
}

#[test]
fn an_empty_stream_leaves_an_empty_spool_and_not_a_missing_one() {
    let path = workspace("empty");

    let spool = SpoolWriter::open(path.clone(), 1024 * 1024).expect("opens");

    assert_eq!(spool.bytes(), 0);
    assert!(path.exists());
}
