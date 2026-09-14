use serde_json::json;
use vigil_model::{Finding, Kind, KnownKind, Severity, Snapshot, State, Subject};

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

pub fn finding_of(key: &str, kind: KnownKind, observed_at: &str) -> Finding {
    let mut record = finding(key, observed_at);
    record.kind = Kind::Known(kind);
    record
}

pub fn snapshot(source: &str, taken_at: &str, port: u64) -> Snapshot {
    Snapshot::new(source, taken_at.to_string()).with(
        format!("tcp|0.0.0.0:{port}"),
        json!({ "protocol": "tcp", "address": "0.0.0.0", "port": port }),
    )
}
