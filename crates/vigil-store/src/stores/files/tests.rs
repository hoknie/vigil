use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use super::limits::Limits;
use super::store::FileStore;
use crate::{Recorded, Store, conformance};

fn temporary_directory(name: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "vigil-store-{}-{name}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&directory);
    directory
}

#[test]
fn it_satisfies_the_contract_every_backend_has_to_satisfy() {
    conformance::run_all(&|| {
        Box::new(FileStore::open(&temporary_directory("conformance")).expect("opens"))
    });
}

#[test]
fn a_restart_remembers_the_baseline_and_the_history() {
    let directory = temporary_directory("restart");
    let key = "port.listen|tcp|0.0.0.0:4444";

    {
        let store = FileStore::open(&directory).expect("opens");
        store
            .set_baseline(&conformance::snapshot(
                "ports",
                "2026-09-09T10:00:00.000Z",
                443,
            ))
            .expect("writes");
        store
            .record(&conformance::finding(key, "2026-09-09T10:00:00.000Z"))
            .expect("records");
    }

    let store = FileStore::open(&directory).expect("reopens");

    let baseline = store.baseline("ports").expect("readable").expect("present");
    assert_eq!(baseline.taken_at, "2026-09-09T10:00:00.000Z");
    assert_eq!(
        store
            .record(&conformance::finding(key, "2026-09-09T11:00:00.000Z"))
            .expect("records"),
        Recorded::Repeated {
            occurrences: 2,
            first_seen_at: "2026-09-09T10:00:00.000Z".into()
        },
        "a finding seen before the restart is not a new finding after it"
    );

    let _ = fs::remove_dir_all(&directory);
}

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
