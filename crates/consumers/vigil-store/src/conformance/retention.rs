use super::samples::finding;
use crate::Store;

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
