use crate::{
    AgentStatus, CollectorRefusal, FindingsSummary, Host, Producer, ProtocolError, Response,
    Snapshot,
};

fn host() -> Host {
    Host {
        host_id: "1c9d8e7b4a5c6d0e".into(),
        install_id: "0199a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b".into(),
        boot_id: "boot".into(),
        hostname: "app-01".into(),
        fqdn: None,
        os: crate::Os {
            family: "linux".into(),
            distro: "alpine".into(),
            version: "3.22".into(),
            kernel: "6.6.0".into(),
            arch: "x86_64".into(),
        },
        addresses: Vec::new(),
        tags: Default::default(),
        peer: None,
    }
}

fn status() -> Response {
    Response::Status {
        schema_version: crate::SCHEMA_VERSION,
        sent_at: "2026-09-09T09:00:00.000Z".into(),
        producer: Producer {
            name: "vigil".into(),
            version: "0.1.0".into(),
        },
        host: Box::new(host()),
        agent: Box::new(AgentStatus {
            configuration_path: Some("/etc/vigil/vigil.yaml".to_string()),
            version: "0.1.0".into(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            collectors: Vec::new(),
            reporters: Vec::new(),
            findings: FindingsSummary::default(),
            silence: crate::Silence::default(),
            budget: crate::AgentBudget::default(),
            store: None,
            buffers: None,
            limitations: vec!["findings are kept in memory".into()],
        }),
    }
}

#[test]
fn an_answer_round_trips_through_its_line() {
    let line = status().to_line();

    assert!(line.ends_with('\n'));
    assert!(line.contains("\"reply\":\"status\""), "{line}");
    match Response::parse(line.trim_end()).expect("parses") {
        Response::Status { host, agent, .. } => {
            assert_eq!(host.hostname, "app-01");
            assert_eq!(agent.interval_seconds, 30);
        }
        other => panic!("came back as {other:?}"),
    }
}

#[test]
fn a_collector_that_has_not_read_yet_is_not_an_empty_host() {
    let never_read = Response::Snapshot {
        collector: "network".into(),
        snapshot: None,
        refusal: None,
    };
    let empty = Response::Snapshot {
        collector: "network".into(),
        snapshot: Some(Snapshot::new("network", "2026-09-09T09:00:00.000Z")),
        refusal: None,
    };

    assert_ne!(never_read.to_line(), empty.to_line());
    assert!(
        !never_read.to_line().contains("\"snapshot\":"),
        "the field is absent, not null: {}",
        never_read.to_line()
    );
}

#[test]
fn a_collector_that_could_not_read_is_not_a_collector_whose_turn_has_not_come() {
    let waiting = Response::Snapshot {
        collector: "launches".into(),
        snapshot: None,
        refusal: None,
    };
    let unable = Response::Snapshot {
        collector: "launches".into(),
        snapshot: None,
        refusal: Some(CollectorRefusal::new(
            crate::CollectorState::Unavailable,
            "auditd is not running",
        )),
    };

    assert_ne!(waiting.to_line(), unable.to_line());
    assert!(
        !waiting.to_line().contains("\"refusal\":"),
        "silence about a refusal is the absence of one: {}",
        waiting.to_line()
    );
    assert!(
        unable.to_line().contains("auditd is not running"),
        "and the reason travels in the daemon's own words: {}",
        unable.to_line()
    );
}

#[test]
fn a_console_built_before_a_refusal_had_a_name_still_reads_an_answer_that_carries_one() {
    let from_a_newer_daemon = r#"{
        "reply": "snapshot", "collector": "launches",
        "refusal": {"state": "unavailable", "reason": "auditd is not running"},
        "something_later_still": 7
    }"#;

    match Response::parse(from_a_newer_daemon).expect("reads") {
        Response::Snapshot {
            collector,
            snapshot,
            refusal,
        } => {
            assert_eq!(collector, "launches");
            assert!(snapshot.is_none());
            assert_eq!(refusal.expect("carried").reason, "auditd is not running");
        }
        other => panic!("came back as {other:?}"),
    }
}

#[test]
fn a_console_reading_a_daemon_older_than_the_field_is_told_nothing_rather_than_refused() {
    let from_an_older_daemon = r#"{"reply": "snapshot", "collector": "ports"}"#;

    match Response::parse(from_an_older_daemon).expect("reads") {
        Response::Snapshot { refusal, .. } => assert!(
            refusal.is_none(),
            "a daemon that cannot say why is not a daemon saying no"
        ),
        other => panic!("came back as {other:?}"),
    }
}

#[test]
fn what_the_daemon_did_on_the_host_comes_back_row_by_row_and_not_as_a_count() {
    let line = Response::Killed {
        report: Box::new(crate::KillReport {
            target: crate::KillTarget::Socket,
            killing: crate::Killing::Terminate,
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            killed: vec![
                crate::Killed::done("tcp|0.0.0.0:4444", 30211, None, "signalled"),
                crate::Killed::refused("tcp|0.0.0.0:80", "no process holds it any more"),
            ],
        }),
    }
    .to_line();

    match Response::parse(line.trim_end()).expect("parses") {
        Response::Killed { report } => {
            assert_eq!(report.done(), 1);
            assert_eq!(report.killed[1].said, "no process holds it any more");
        }
        other => panic!("came back as {other:?}"),
    }
}

#[test]
fn what_the_daemon_did_to_what_starts_by_itself_comes_back_row_by_row_and_not_as_a_count() {
    let line = Response::Controlled {
        report: Box::new(crate::ControlReport {
            target: crate::ControlTarget::Unit,
            controlling: crate::Controlling::Stop,
            acted_at: "2026-09-16T10:00:00.000Z".into(),
            controlled: vec![
                crate::Controlled::done(
                    "unit|nginx.service",
                    Some("nginx.service".into()),
                    "systemctl stop nginx.service",
                ),
                crate::Controlled::refused(
                    "unit|dbus.service",
                    "this unit is not in the last reading",
                ),
            ],
        }),
    }
    .to_line();

    match Response::parse(line.trim_end()).expect("parses") {
        Response::Controlled { report } => {
            assert_eq!(report.done(), 1);
            assert_eq!(
                report.controlled[1].said,
                "this unit is not in the last reading"
            );
        }
        other => panic!("came back as {other:?}"),
    }
}

#[test]
fn a_console_older_than_the_verb_reads_the_answer_to_it_as_an_answer_it_does_not_know() {
    let unknown = r#"{"reply": "something_a_later_daemon_does", "what": 1}"#;

    assert!(
        Response::parse(unknown).is_err(),
        "an answer whose shape this build cannot name is a refusal to guess, not a \
         silently empty screen"
    );
}

#[test]
fn a_refusal_travels_as_an_answer_rather_than_as_a_closed_connection() {
    let line = Response::Error {
        error: ProtocolError::new(ProtocolError::UNKNOWN_COLLECTOR, "no collector \"files\""),
    }
    .to_line();

    match Response::parse(line.trim_end()).expect("parses") {
        Response::Error { error } => assert_eq!(error.code, ProtocolError::UNKNOWN_COLLECTOR),
        other => panic!("came back as {other:?}"),
    }
}
