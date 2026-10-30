use serde_json::{Value, json};

use crate::{Finding, Kind, KnownKind, Severity, State, Subject};

fn finding(kind: KnownKind, key: Value, before: Option<Value>, after: Option<Value>) -> Finding {
    Finding {
        event_id: "0192f3c2-0d17-7a55-b0c9-6e2b8f41a920".into(),
        finding_key: "key".into(),
        kind: Kind::Known(kind),
        severity: Severity::High,
        state: State::Open,
        observed_at: "2026-10-30T10:00:00.000Z".into(),
        first_seen_at: "2026-10-30T10:00:00.000Z".into(),
        occurrences: 1,
        title: "something".into(),
        subject: Subject {
            object: "socket".into(),
            key,
        },
        before,
        after,
        evidence: Vec::new(),
        redacted: Vec::new(),
        rule: None,
        labels: Default::default(),
    }
}

fn socket() -> Value {
    json!({"protocol": "udp", "address": "0.0.0.0", "port": 41234})
}

fn held_by(exe: &str) -> Value {
    json!({"protocol": "udp", "port": 41234, "process": {"pid": 812, "exe": exe}})
}

#[test]
fn a_new_socket_belongs_to_the_program_that_holds_it() {
    let new = finding(
        KnownKind::PortListenNew,
        socket(),
        None,
        Some(held_by("/usr/bin/netbird")),
    );

    assert_eq!(new.program(), Some("/usr/bin/netbird"));
}

#[test]
fn a_socket_that_went_away_belongs_to_the_program_that_held_it() {
    let gone = finding(
        KnownKind::PortListenRemoved,
        socket(),
        Some(held_by("/usr/bin/netbird")),
        None,
    );

    assert_eq!(gone.program(), Some("/usr/bin/netbird"));
}

#[test]
fn a_socket_taken_over_belongs_to_its_new_owner_so_silencing_the_old_one_does_not_hide_the_takeover()
 {
    let taken = finding(
        KnownKind::PortListenOwnerChanged,
        socket(),
        Some(held_by("/usr/bin/netbird")),
        Some(held_by("/tmp/.x/nc")),
    );

    assert_eq!(taken.program(), Some("/tmp/.x/nc"));
}

#[test]
fn a_new_owner_nobody_could_name_is_no_program_rather_than_the_old_one() {
    let taken = finding(
        KnownKind::PortListenOwnerChanged,
        socket(),
        Some(held_by("/usr/bin/netbird")),
        Some(json!({"protocol": "udp", "port": 41234, "process": null})),
    );

    assert_eq!(taken.program(), None);
}

#[test]
fn a_launch_belongs_to_the_executable_it_names() {
    let launch = finding(
        KnownKind::ExecFirstSeenForUser,
        json!({"executable": "/usr/local/bin/netbird", "user": "alice"}),
        None,
        None,
    );

    assert_eq!(launch.program(), Some("/usr/local/bin/netbird"));
}

#[test]
fn a_finding_about_a_file_has_no_program() {
    let file = finding(
        KnownKind::FileChanged,
        json!({"path": "/etc/netbird/config.json"}),
        Some(json!({"sha256": "a"})),
        Some(json!({"sha256": "b"})),
    );

    assert_eq!(file.program(), None);
}
