use serde_json::json;
use vigil_collect::Health;
use vigil_model::{Finding, Host, Kind, Os, Severity, Snapshot, State as FindingState, Subject};

use super::State;
use crate::types::{Reading, Startup};

pub fn host() -> Host {
    Host {
        host_id: "1c9d8e7b4a5c6d0e".into(),
        install_id: "0199a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b".into(),
        boot_id: "boot-id".into(),
        hostname: "app-01".into(),
        fqdn: None,
        os: Os {
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

pub fn state() -> State {
    State::new(
        Startup {
            host: host(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            periods: [("ports".to_string(), 30u32)].into_iter().collect(),
        },
        &[("ports", Health::Ok)],
        &["ndjson".to_string()],
        &[],
    )
}

pub fn reading(snapshot: Snapshot) -> Reading {
    Reading {
        collector: "ports",
        at: "2026-09-09T09:00:00.000Z".into(),
        duration_ms: 5,
        every_seconds: 30,
        next_run_at: "2026-09-09T09:00:30.000Z".into(),
        skipped: 0,
        snapshot,
    }
}

pub fn snapshot() -> Snapshot {
    Snapshot::new("ports", "2026-09-09T09:00:00.000Z").with(
        "tcp|0.0.0.0:4444",
        json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": 4444, "uid": 33, "user": "www-data",
            "process": {
                "exe": "/tmp/.x/nc", "exe_deleted": true,
                "cmdline": "nc -l -p 4444", "cmdline_redacted": false,
            },
            "owner_resolved": true,
        }),
    )
}

pub fn finding(title: &str) -> Finding {
    Finding {
        event_id: format!("event-{title}"),
        finding_key: format!("port.listen|tcp|0.0.0.0:{}", title.len()),
        kind: Kind::from("port.listen.new".to_string()),
        severity: Severity::Critical,
        state: FindingState::Open,
        observed_at: "2026-09-09T09:00:00.000Z".into(),
        first_seen_at: "2026-09-09T09:00:00.000Z".into(),
        occurrences: 1,
        title: title.to_string(),
        subject: Subject {
            object: "socket".into(),
            key: json!({"protocol": "tcp", "address": "0.0.0.0", "port": 4444}),
        },
        before: None,
        after: None,
        evidence: Vec::new(),
        redacted: Vec::new(),
        rule: Some("new_listening_port".into()),
        labels: Default::default(),
    }
}
