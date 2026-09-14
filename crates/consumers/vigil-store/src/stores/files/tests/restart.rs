use std::fs;

use super::harness::temporary_directory;
use crate::stores::files::FileStore;
use crate::{Recorded, Store, conformance};

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
