use vigil_model::{Kind, KnownKind};

use super::samples::{finding, finding_of};
use crate::{Recorded, Store};

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

pub fn one_open_finding_is_found_by_its_object_and_its_kind(store: &dyn Store) {
    let key = "agent.collector|launches";
    store
        .record(&finding_of(
            key,
            KnownKind::AgentCollectorDegraded,
            "2026-09-09T10:00:00.000Z",
        ))
        .expect("recorded");

    let open = store
        .open_finding(key, "agent.collector.degraded")
        .expect("readable")
        .expect("the finding that was recorded");

    assert_eq!(open.first_seen_at, "2026-09-09T10:00:00.000Z");
    assert!(
        store
            .open_finding(key, "port.listen.new")
            .expect("readable")
            .is_none(),
        "one object can carry a finding of one kind and none of another"
    );
    assert!(
        store
            .open_finding("agent.collector|ports", "agent.collector.degraded")
            .expect("readable")
            .is_none(),
        "and asking about an object with no history is not an error"
    );

    store
        .resolve(key, "agent.collector.degraded", "2026-09-09T11:00:00.000Z")
        .expect("resolves");
    assert!(
        store
            .open_finding(key, "agent.collector.degraded")
            .expect("readable")
            .is_none(),
        "a finding that was closed is not one still standing, and a restart that reopens it \
         tells an operator about trouble that ended"
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
