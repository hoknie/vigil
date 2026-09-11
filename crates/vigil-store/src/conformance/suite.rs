use serde_json::json;
use vigil_model::{Finding, Kind, KnownKind, Severity, Snapshot, State, Subject};

use crate::{Recorded, Store};

pub fn finding(key: &str, observed_at: &str) -> Finding {
    Finding {
        event_id: format!("event-for-{key}-at-{observed_at}"),
        finding_key: key.to_string(),
        kind: Kind::Known(KnownKind::PortListenNew),
        severity: Severity::Medium,
        state: State::Open,
        observed_at: observed_at.to_string(),
        first_seen_at: observed_at.to_string(),
        occurrences: 1,
        title: format!("something happened to {key}"),
        subject: Subject {
            object: "socket".into(),
            key: json!({ "key": key }),
        },
        before: None,
        after: None,
        evidence: Vec::new(),
        redacted: Vec::new(),
        rule: Some("conformance".into()),
        labels: Default::default(),
    }
}

pub fn snapshot(source: &str, taken_at: &str, port: u64) -> Snapshot {
    Snapshot::new(source, taken_at.to_string()).with(
        format!("tcp|0.0.0.0:{port}"),
        json!({ "protocol": "tcp", "address": "0.0.0.0", "port": port }),
    )
}

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

pub fn a_repeat_raises_the_counter_and_keeps_the_first_sighting(store: &dyn Store) {
    let key = "port.listen|tcp|0.0.0.0:4444";

    assert_eq!(
        store
            .record(&finding(key, "2026-09-09T10:00:00.000Z"))
            .expect("recorded"),
        Recorded::Created
    );
    assert_eq!(
        store
            .record(&finding(key, "2026-09-09T10:05:00.000Z"))
            .expect("recorded"),
        Recorded::Repeated {
            occurrences: 2,
            first_seen_at: "2026-09-09T10:00:00.000Z".into()
        }
    );
    assert_eq!(
        store
            .record(&finding(key, "2026-09-09T10:10:00.000Z"))
            .expect("recorded"),
        Recorded::Repeated {
            occurrences: 3,
            first_seen_at: "2026-09-09T10:00:00.000Z".into()
        }
    );

    let open = store.open_findings(10).expect("readable");
    let recorded = open.iter().find(|f| f.finding_key == key).expect("present");

    assert_eq!(recorded.occurrences, 3);
    assert_eq!(
        recorded.first_seen_at, "2026-09-09T10:00:00.000Z",
        "the first sighting is the one the store remembers, not the latest one"
    );
    assert_eq!(recorded.observed_at, "2026-09-09T10:10:00.000Z");
    assert_eq!(open.iter().filter(|f| f.finding_key == key).count(), 1);
}

pub fn a_different_kind_about_the_same_object_is_not_a_repeat(store: &dyn Store) {
    let key = "port.listen|tcp|0.0.0.0:7009";

    let opened = finding(key, "2026-09-09T10:00:00.000Z");
    let mut closed = finding(key, "2026-09-09T10:01:00.000Z");
    closed.kind = Kind::Known(KnownKind::PortListenRemoved);

    assert_eq!(store.record(&opened).expect("recorded"), Recorded::Created);
    assert_eq!(
        store.record(&closed).expect("recorded"),
        Recorded::Created,
        "a closure is not a repeat of the opening it closes"
    );

    let open = store.open_findings(10).expect("readable");
    assert_eq!(open.len(), 2, "both statements have to survive: {open:?}");
}

pub fn a_resolved_finding_stops_being_open(store: &dyn Store) {
    let key = "port.listen|tcp|0.0.0.0:4444";
    store
        .record(&finding(key, "2026-09-09T10:00:00.000Z"))
        .expect("recorded");

    let closed = store
        .resolve(key, "port.listen.new", "2026-09-09T10:30:00.000Z")
        .expect("resolves");

    assert!(closed);
    assert!(
        store.open_findings(10).expect("readable").is_empty(),
        "a resolved finding must leave the open list"
    );
    assert!(
        !store
            .resolve(key, "port.listen.new", "2026-09-09T11:00:00.000Z")
            .expect("resolves"),
        "closing an already closed finding must not write a second time"
    );
}

pub fn resolving_something_that_was_never_recorded_is_not_an_error(store: &dyn Store) {
    assert!(
        !store
            .resolve(
                "port.listen|tcp|0.0.0.0:22",
                "port.listen.new",
                "2026-09-09T10:00:00.000Z"
            )
            .expect("answers"),
        "an unmatched removal is a fact about history, not a failure"
    );
}

pub fn open_findings_come_back_newest_first_and_within_the_limit(store: &dyn Store) {
    for (index, key) in ["a", "b", "c", "d"].iter().enumerate() {
        store
            .record(&finding(key, &format!("2026-09-09T10:0{index}:00.000Z")))
            .expect("recorded");
    }

    let open = store.open_findings(2).expect("readable");

    assert_eq!(open.len(), 2, "the limit is a limit");
    assert_eq!(open[0].finding_key, "d");
    assert_eq!(open[1].finding_key, "c");
}

pub fn pruning_drops_the_old_and_keeps_the_rest(store: &dyn Store) {
    store
        .record(&finding("old", "2026-06-01T10:00:00.000Z"))
        .expect("recorded");
    store
        .record(&finding("recent", "2026-09-09T10:00:00.000Z"))
        .expect("recorded");

    let dropped = store.prune("2026-09-01T00:00:00.000Z").expect("prunes");

    assert_eq!(dropped, 1);
    let left = store.open_findings(10).expect("readable");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].finding_key, "recent");
}

pub fn a_store_says_what_it_is_holding_and_what_it_threw_away(store: &dyn Store) {
    let empty = store.kept().expect("answers");
    assert!(
        empty.is_empty() && empty.oldest_at.is_none(),
        "a store with nothing in it answers empty, never zero bytes of something: {empty:?}"
    );

    store
        .record(&finding("old", "2026-06-01T10:00:00.000Z"))
        .expect("recorded");
    store
        .record(&finding("recent", "2026-09-09T10:00:00.000Z"))
        .expect("recorded");

    let holding = store.kept().expect("answers");
    assert_eq!(holding.records.held, 2);
    assert!(
        holding.records.ceiling >= holding.records.held,
        "a ceiling under what is held is not a ceiling: {holding:?}"
    );
    assert_eq!(
        holding.oldest_at.as_deref(),
        Some("2026-06-01T10:00:00.000Z")
    );
}

pub fn a_store_says_how_much_of_its_history_is_still_open(store: &dyn Store) {
    let closed_later = "port.listen|tcp|0.0.0.0:4444";
    store
        .record(&finding(closed_later, "2026-09-09T10:00:00.000Z"))
        .expect("recorded");
    store
        .record(&finding(
            "port.listen|tcp|0.0.0.0:8080",
            "2026-09-09T10:01:00.000Z",
        ))
        .expect("recorded");

    store
        .resolve(closed_later, "port.listen.new", "2026-09-09T10:30:00.000Z")
        .expect("resolves");

    let kept = store.kept().expect("answers");
    assert_eq!(
        kept.records.held, 2,
        "a closed finding is history and stays kept: {kept:?}"
    );
    assert_eq!(
        kept.open, 1,
        "a closed finding is kept and is not open: {kept:?}"
    );
    assert_eq!(
        kept.open as usize,
        store.open_findings(100).expect("readable").len(),
        "the count of what is open and the list of it are one answer, not two"
    );
}

pub fn what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it(store: &dyn Store) {
    store
        .record(&finding("old", "2026-06-01T10:00:00.000Z"))
        .expect("recorded");
    store
        .record(&finding("recent", "2026-09-09T10:00:00.000Z"))
        .expect("recorded");

    store.prune("2026-09-01T00:00:00.000Z").expect("prunes");

    let kept = store.kept().expect("answers");
    assert_eq!(
        kept.dropped.past_the_window, 1,
        "the retention window threw one record away and did not say so: {kept:?}"
    );
    assert_eq!(
        kept.dropped.at_the_ceiling, 0,
        "and it was not the size ceiling that did it: {kept:?}"
    );
    assert_eq!(
        kept.oldest_at.as_deref(),
        Some("2026-09-09T10:00:00.000Z"),
        "the oldest record is the oldest one still kept"
    );
}

pub fn run_all(new_store: &dyn Fn() -> Box<dyn Store>) {
    a_collector_with_no_history_has_no_baseline(new_store().as_ref());
    the_baseline_is_the_last_reading(new_store().as_ref());
    baselines_are_kept_per_collector(new_store().as_ref());
    a_forgotten_baseline_is_gone_and_takes_nothing_else_with_it(new_store().as_ref());
    a_repeat_raises_the_counter_and_keeps_the_first_sighting(new_store().as_ref());
    a_different_kind_about_the_same_object_is_not_a_repeat(new_store().as_ref());
    a_resolved_finding_stops_being_open(new_store().as_ref());
    resolving_something_that_was_never_recorded_is_not_an_error(new_store().as_ref());
    open_findings_come_back_newest_first_and_within_the_limit(new_store().as_ref());
    pruning_drops_the_old_and_keeps_the_rest(new_store().as_ref());
    a_store_says_what_it_is_holding_and_what_it_threw_away(new_store().as_ref());
    a_store_says_how_much_of_its_history_is_still_open(new_store().as_ref());
    what_a_ceiling_threw_away_is_named_by_the_ceiling_that_threw_it(new_store().as_ref());
}
