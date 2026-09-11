use serde_json::json;
use vigil_model::Snapshot;

use super::harness::drawn_at;
use crate::ui::{Reading, Subject, fixture};

fn reading(items: Vec<(&str, serde_json::Value)>) -> Reading {
    let mut snapshot = Snapshot::new("users", "2026-09-09T09:00:00.000Z");
    for (key, item) in items {
        snapshot = snapshot.with(key, item);
    }
    Reading::Taken(snapshot)
}

fn source(name: &str, path: &str, present: bool, read: bool, sessions: u64) -> serde_json::Value {
    json!({
        "source": name, "path": path, "present": present, "read": read,
        "answers": present && read, "sessions": sessions, "reason": null,
    })
}

#[test]
fn a_session_two_sources_saw_is_one_row_that_names_them_both() {
    let page = drawn_at(&fixture::view(), Subject::LoggedIn, 140);

    assert_eq!(
        page.matches("pts/0").count(),
        1,
        "one login must not be two rows: {page}"
    );
    assert!(page.contains("logind, utmp"), "{page}");
}

#[test]
fn a_session_only_one_source_saw_says_which_one_that_was() {
    let page = drawn_at(&fixture::view(), Subject::LoggedIn, 140);

    assert!(
        page.contains("background"),
        "a session nobody is sitting at is still a way in: {page}"
    );
    assert!(page.contains("closing"), "{page}");
}

#[test]
fn the_sources_of_logins_are_rows_of_their_own_under_the_people() {
    let page = drawn_at(&fixture::view(), Subject::LoggedIn, 140);

    assert!(page.contains("/run/systemd/sessions"), "{page}");
    assert!(page.contains("/run/utmp"), "{page}");
    assert!(page.contains("not on this host"), "{page}");
    assert!(page.contains("login source"), "{page}");
}

#[test]
fn a_host_whose_sources_answer_and_hold_nobody_does_not_read_like_a_host_with_no_sources() {
    let mut quiet = fixture::view();
    quiet.readings.put(
        "users",
        reading(vec![
            ("account|root", json!({"name": "root", "uid": 0})),
            (
                "session-source|logind",
                source("logind", "/run/systemd/sessions", true, true, 0),
            ),
            (
                "session-source|utmp",
                source("utmp", "/run/utmp", false, false, 0),
            ),
        ]),
    );
    let mut blind = fixture::view();
    blind.readings.put(
        "users",
        reading(vec![
            ("account|root", json!({"name": "root", "uid": 0})),
            (
                "session-source|logind",
                source("logind", "/run/systemd/sessions", false, false, 0),
            ),
            (
                "session-source|utmp",
                source("utmp", "/run/utmp", false, false, 0),
            ),
        ]),
    );

    let quiet = drawn_at(&quiet, Subject::LoggedIn, 140);
    let blind = drawn_at(&blind, Subject::LoggedIn, 140);

    assert!(quiet.contains("logind: read, 0"), "{quiet}");
    assert!(blind.contains("logind: not on this host"), "{blind}");
    assert_ne!(
        quiet, blind,
        "'we looked and nobody is logged in' and 'we have nowhere to look' are two answers"
    );
}

#[test]
fn the_tally_counts_the_people_and_not_the_places_their_logins_were_read_from() {
    let page = drawn_at(&fixture::view(), Subject::LoggedIn, 140);

    assert!(page.contains("3 sessions"), "{page}");
    assert!(page.contains("2 with somebody at a terminal"), "{page}");
}
