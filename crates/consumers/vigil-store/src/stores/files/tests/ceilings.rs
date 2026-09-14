use std::fs;

use super::harness::temporary_directory;
use crate::stores::files::{FileStore, Limits};
use crate::{Store, conformance};

#[test]
fn overflow_drops_the_oldest_and_never_refuses_the_newest() {
    let directory = temporary_directory("cap");
    let store = FileStore::with_limits(
        &directory,
        Limits {
            findings: 3,
            journal_bytes: 16 * 1024 * 1024,
        },
    )
    .expect("opens");

    for index in 0..6 {
        store
            .record(&conformance::finding(
                &format!("key-{index}"),
                &format!("2026-09-09T10:0{index}:00.000Z"),
            ))
            .expect("a full store still records");
    }

    let open = store.open_findings(10).expect("readable");
    let keys: Vec<&str> = open.iter().map(|f| f.finding_key.as_str()).collect();

    assert_eq!(
        keys,
        vec!["key-5", "key-4", "key-3"],
        "the oldest three went"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn what_the_ceiling_threw_away_is_counted_and_not_only_rewritten() {
    let directory = temporary_directory("counted");
    let store = FileStore::with_limits(
        &directory,
        Limits {
            findings: 3,
            journal_bytes: 16 * 1024 * 1024,
        },
    )
    .expect("opens");

    for index in 0..6 {
        store
            .record(&conformance::finding(
                &format!("key-{index}"),
                &format!("2026-09-09T10:0{index}:00.000Z"),
            ))
            .expect("records");
    }
    store.prune("2026-09-09T10:05:00.000Z").expect("prunes");

    let kept = store.kept().expect("answers");

    assert!(
        kept.dropped.at_the_ceiling > 0,
        "the ceiling threw records away and said nothing: {kept:?}"
    );
    assert!(
        kept.dropped.past_the_window > 0,
        "the retention window threw records away and said nothing: {kept:?}"
    );
    assert_ne!(
        kept.dropped.at_the_ceiling, kept.dropped.past_the_window,
        "two ceilings counted as one is a number nobody can act on"
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn a_full_store_does_not_rewrite_its_journal_on_every_write() {
    let directory = temporary_directory("hysteresis");
    let store = FileStore::with_limits(
        &directory,
        Limits {
            findings: 100,
            journal_bytes: 16 * 1024 * 1024,
        },
    )
    .expect("opens");

    for index in 0..300 {
        store
            .record(&conformance::finding(
                &format!("key-{index:04}"),
                &format!("2026-09-09T10:00:{:02}.000Z", index % 60),
            ))
            .expect("records");
    }

    assert!(
        store.compactions() <= 30,
        "200 findings past the cap caused {} rewrites; one per write is the bug",
        store.compactions()
    );
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn a_journal_over_its_byte_cap_is_compacted_rather_than_closed() {
    let directory = temporary_directory("bytes");
    let store = FileStore::with_limits(
        &directory,
        Limits {
            findings: 10_000,
            journal_bytes: 1_024,
        },
    )
    .expect("opens");

    for hour in 0..40 {
        store
            .record(&conformance::finding(
                "port.listen|tcp|0.0.0.0:4444",
                &format!("2026-09-09T{hour:02}:00:00.000Z"),
            ))
            .expect("records");
    }

    let open = store.open_findings(10).expect("readable");
    assert_eq!(open.len(), 1);
    assert_eq!(
        open[0].occurrences, 40,
        "compaction must not lose the count"
    );

    let journal =
        fs::read_to_string(directory.join("findings").join("journal.ndjson")).expect("readable");
    assert!(
        journal.lines().count() <= 3,
        "the journal grew with sightings instead of being compacted: {} lines",
        journal.lines().count()
    );
    let _ = fs::remove_dir_all(&directory);
}
