use std::fs;
use std::time::Instant;

use super::harness::temporary_directory;
use crate::stores::files::FileStore;
use crate::{Store, conformance};

#[test]
fn the_oldest_record_is_the_oldest_one_still_kept_and_not_the_oldest_ever_written() {
    let directory = temporary_directory("oldest");
    let store = FileStore::open(&directory).expect("opens");

    store
        .record(&conformance::finding("old", "2026-06-01T10:00:00.000Z"))
        .expect("records");
    store
        .record(&conformance::finding("recent", "2026-09-09T10:00:00.000Z"))
        .expect("records");
    assert_eq!(
        store.kept().expect("answers").oldest_at.as_deref(),
        Some("2026-06-01T10:00:00.000Z")
    );

    store.prune("2026-09-01T00:00:00.000Z").expect("prunes");

    assert_eq!(
        store.kept().expect("answers").oldest_at.as_deref(),
        Some("2026-09-09T10:00:00.000Z"),
        "the record that was pruned is not the age of this store"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn a_record_seen_again_moves_the_age_of_the_store_with_it() {
    let directory = temporary_directory("moved");
    let store = FileStore::open(&directory).expect("opens");

    store
        .record(&conformance::finding("one", "2026-06-01T10:00:00.000Z"))
        .expect("records");
    store
        .record(&conformance::finding("one", "2026-09-09T10:00:00.000Z"))
        .expect("records the repeat");

    assert_eq!(
        store.kept().expect("answers").oldest_at.as_deref(),
        Some("2026-09-09T10:00:00.000Z"),
        "one record was rewritten, so the moment it was last seen is the only one left"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn a_store_with_nothing_in_it_says_so_and_does_not_answer_zero_bytes() {
    let directory = temporary_directory("empty");
    let store = FileStore::open(&directory).expect("opens");

    let kept = store.kept().expect("answers");

    assert!(kept.is_empty());
    assert_eq!(kept.oldest_at, None);
    assert!(
        kept.records.ceiling > 0 && kept.bytes.ceiling > 0,
        "the ceilings exist whether or not anything is under them: {kept:?}"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn the_age_of_the_oldest_record_costs_nothing_to_ask_for() {
    let directory = temporary_directory("cheap");
    let store = FileStore::open(&directory).expect("opens");
    for index in 0..300 {
        store
            .record(&conformance::finding(
                &format!("key-{index:04}"),
                &format!("2026-09-09T10:00:{:02}.000Z", index % 60),
            ))
            .expect("records");
    }

    let started = Instant::now();
    for _ in 0..100 {
        store.kept().expect("answers");
    }
    let took = started.elapsed();

    assert!(
        took.as_millis() < 50,
        "a hundred answers took {took:?}: the age is being recomputed by walking the records"
    );
    let _ = fs::remove_dir_all(&directory);
}
