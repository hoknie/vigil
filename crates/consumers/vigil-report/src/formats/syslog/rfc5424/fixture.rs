use std::collections::BTreeMap;

use serde_json::json;
use vigil_model::{Evidence, Finding, Host, Kind, KnownKind, Os, Severity, State, Subject};

use super::Rfc5424;
use crate::formats::syslog::SyslogFacility;

pub(super) fn host() -> Host {
    Host {
        host_id: "b7f1c0a2".into(),
        install_id: "0192e7aa-1c02-7f10-8d44-9a1b6c2e5f03".into(),
        boot_id: "6f2c".into(),
        hostname: "web-03".into(),
        fqdn: Some("web-03.example.com".into()),
        os: Os {
            family: "linux".into(),
            distro: "debian".into(),
            version: "12".into(),
            kernel: "6.1.0-18-amd64".into(),
            arch: "x86_64".into(),
        },
        addresses: vec!["10.0.0.13".into()],
        tags: BTreeMap::new(),
        peer: None,
    }
}

pub(super) fn finding() -> Finding {
    Finding {
        event_id: "0192f3c1-8a44-7c1e-9b31-2f5c0a3d77e1".into(),
        finding_key: "port.listen|tcp|0.0.0.0:4444".into(),
        kind: Kind::Known(KnownKind::PortListenNew),
        severity: Severity::High,
        state: State::Open,
        observed_at: "2026-09-09T12:04:02.311Z".into(),
        first_seen_at: "2026-09-09T12:04:02.311Z".into(),
        occurrences: 1,
        title: "New listening socket tcp 0.0.0.0:4444".into(),
        subject: Subject {
            object: "socket".into(),
            key: json!({"protocol": "tcp", "address": "0.0.0.0", "port": 4444}),
        },
        before: None,
        after: Some(json!({"protocol": "tcp", "process": {"pid": 4711, "exe": "/tmp/nc"}})),
        evidence: vec![
            Evidence {
                kind: "path".into(),
                value: "/tmp/nc".into(),
            },
            Evidence {
                kind: "cmdline".into(),
                value: "nc -l 4444".into(),
            },
        ],
        redacted: Vec::new(),
        rule: Some("new_listening_port".into()),
        labels: BTreeMap::new(),
    }
}

pub(super) fn format() -> Rfc5424 {
    Rfc5424::new(
        SyslogFacility::parse("local4").expect("known facility"),
        "vigil",
        4242,
    )
}
