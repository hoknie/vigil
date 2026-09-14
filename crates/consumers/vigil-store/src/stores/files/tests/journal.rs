use std::fs;
use std::time::Instant;

use super::harness::temporary_directory;
use crate::stores::files::{FileStore, Limits};
use crate::{Store, conformance};

#[test]
fn the_journal_names_the_file_an_operator_would_run_jq_over() {
    let directory = temporary_directory("path");
    let store = FileStore::open(&directory).expect("opens");

    let path = store.journal_path().expect("a path");

    assert!(path.ends_with("findings/journal.ndjson"), "{path:?}");
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn opening_a_journal_filled_to_its_ceiling_does_not_hold_up_the_start() {
    let directory = temporary_directory("full");
    let limits = Limits::default();
    fs::create_dir_all(directory.join("findings")).expect("makes the directory");
    let journal: String = (0..limits.findings)
        .map(|index| {
            let record = conformance::finding(
                &format!("port.listen|tcp|0.0.0.0:{index:05}"),
                &format!("2026-09-09T10:00:00.{:03}Z", index % 1_000),
            );
            format!("{}\n", serde_json::to_string(&record).expect("serialises"))
        })
        .collect();
    fs::write(directory.join("findings").join("journal.ndjson"), journal).expect("writes");

    let started = Instant::now();
    let store = FileStore::open(&directory).expect("opens");
    let opening = started.elapsed();

    let started = Instant::now();
    let recalled = store.open_findings(500).expect("readable");
    let recalling = started.elapsed();

    let started = Instant::now();
    let kept = store.kept().expect("answers");
    let counting = started.elapsed();

    assert_eq!(recalled.len(), 500);
    assert_eq!(kept.open, limits.findings as u64);
    assert!(
        counting.as_millis() < 50,
        "counting what is still open in a full journal took {counting:?}"
    );
    assert!(
        opening.as_millis() < 750,
        "a full journal opened in {opening:?} and the start waits for it"
    );
    assert!(
        recalling.as_millis() < 250,
        "the newest 500 of a full journal came back in {recalling:?}"
    );
    let _ = fs::remove_dir_all(&directory);
}
