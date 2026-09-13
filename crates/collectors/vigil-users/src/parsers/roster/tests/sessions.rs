use std::collections::BTreeSet;

use serde_json::Value;

use super::super::reading::AccountsReading;
use super::super::snapshot::accounts_snapshot;
use crate::parsers::sessions::{LOGIND, Session, SessionSource, UTMP};

#[test]
fn a_session_row_says_who_saw_it_and_what_kind_of_session_it_is() {
    let sessions = [Session {
        user: "deploy".into(),
        uid: Some(1000),
        line: "pts/0".into(),
        from: "10.0.0.7".into(),
        remote: true,
        pid: 4021,
        id: "83".into(),
        service: "sshd".into(),
        kind: "tty".into(),
        class: "user".into(),
        state: "active".into(),
        sources: BTreeSet::from([LOGIND, UTMP]),
    }];
    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &[],
            groups: &[],
            shadow: None,
            sudo: &[],
            keys: &[],
            sessions: &sessions,
            session_sources: &[],
        },
    );

    let row = &snapshot.items["session|deploy|pts/0"];
    assert_eq!(row["seen_by"], serde_json::json!(["logind", "utmp"]));
    assert_eq!(row["session_id"], "83");
    assert_eq!(row["service"], "sshd");
    assert_eq!(row["class"], "user");
    assert_eq!(row["state"], "active");
    assert_eq!(row["attended"], true);
    assert_eq!(row["uid"], 1000);
}

#[test]
fn a_host_with_no_login_source_says_so_in_a_row_rather_than_with_an_empty_list() {
    let sources = [
        SessionSource::absent(UTMP, "/run/utmp", "no file at /run/utmp or /var/run/utmp"),
        SessionSource::absent(
            LOGIND,
            "/run/systemd/sessions",
            "systemd-logind keeps no sessions",
        ),
    ];
    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &[],
            groups: &[],
            shadow: None,
            sudo: &[],
            keys: &[],
            sessions: &[],
            session_sources: &sources,
        },
    );

    for name in ["utmp", "logind"] {
        let row = &snapshot.items[&format!("session-source|{name}")];
        assert_eq!(row["present"], false);
        assert_eq!(row["read"], false);
        assert_eq!(row["answers"], false);
        assert!(row["reason"].is_string(), "why is not optional");
    }
}

#[test]
fn a_source_that_was_read_and_held_nothing_is_not_the_same_row_as_one_that_is_not_there() {
    let sources = [
        SessionSource::absent(UTMP, "/run/utmp", "no file there"),
        SessionSource::read(LOGIND, "/run/systemd/sessions", 0),
    ];
    let snapshot = accounts_snapshot(
        "2026-09-09T12:00:00.000Z",
        &AccountsReading {
            passwd: &[],
            groups: &[],
            shadow: None,
            sudo: &[],
            keys: &[],
            sessions: &[],
            session_sources: &sources,
        },
    );

    assert_eq!(snapshot.items["session-source|logind"]["answers"], true);
    assert_eq!(snapshot.items["session-source|logind"]["sessions"], 0);
    assert_eq!(
        snapshot.items["session-source|logind"]["reason"],
        Value::Null
    );
    assert_eq!(snapshot.items["session-source|utmp"]["answers"], false);
}
