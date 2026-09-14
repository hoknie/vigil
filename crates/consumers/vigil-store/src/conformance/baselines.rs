use super::samples::{finding, snapshot};
use crate::Store;

pub fn a_collector_with_no_history_has_no_baseline(store: &dyn Store) {
    assert!(
        store.baseline("ports").expect("readable").is_none(),
        "a store with no history must answer None, never an empty snapshot"
    );
}

pub fn the_baseline_is_the_last_reading(store: &dyn Store) {
    store
        .set_baseline(&snapshot("ports", "2026-09-09T10:00:00.000Z", 22))
        .expect("write");
    store
        .set_baseline(&snapshot("ports", "2026-09-09T10:00:30.000Z", 443))
        .expect("write");

    let baseline = store.baseline("ports").expect("readable").expect("present");

    assert_eq!(baseline.taken_at, "2026-09-09T10:00:30.000Z");
    assert!(baseline.items.contains_key("tcp|0.0.0.0:443"));
    assert!(
        !baseline.items.contains_key("tcp|0.0.0.0:22"),
        "the previous reading must not survive inside the current one"
    );
}

pub fn baselines_are_kept_per_collector(store: &dyn Store) {
    store
        .set_baseline(&snapshot("ports", "2026-09-09T10:00:00.000Z", 22))
        .expect("write");
    store
        .set_baseline(&snapshot("users", "2026-09-09T10:00:00.000Z", 1))
        .expect("write");

    assert_eq!(
        store
            .baseline("ports")
            .expect("readable")
            .expect("present")
            .source,
        "ports"
    );
    assert_eq!(
        store
            .baseline("users")
            .expect("readable")
            .expect("present")
            .source,
        "users"
    );
}

pub fn a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it(store: &dyn Store) {
    store
        .set_baseline(&snapshot("ports", "2026-09-09T10:00:00.000Z", 443))
        .expect("writes");
    store
        .set_baseline(&snapshot("users", "2026-09-09T10:00:00.000Z", 22))
        .expect("writes");
    store
        .record(&finding(
            "port.listen|tcp|0.0.0.0:443",
            "2026-09-09T10:00:00.000Z",
        ))
        .expect("records");

    assert!(store.forget_baseline("ports").expect("forgets"));

    assert!(
        store.baseline("ports").expect("readable").is_none(),
        "a forgotten baseline must read as no baseline, not as an empty one"
    );
    assert!(
        store.baseline("users").expect("readable").is_some(),
        "forgetting one collector must not touch another's"
    );
    assert_eq!(
        store.open_findings(10).expect("readable").len(),
        1,
        "forgetting a reading must not forget what happened"
    );
    assert!(
        !store.forget_baseline("ports").expect("forgets"),
        "forgetting what is not there is not an error, and it says so"
    );
}
