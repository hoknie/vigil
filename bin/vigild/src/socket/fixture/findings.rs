use serde_json::json;
use vigil_model::{Finding, Kind, Severity, State as FindingState, Subject};

pub fn finding(title: &str) -> Finding {
    finding_of(Severity::Critical, title)
}

pub fn finding_of(severity: Severity, title: &str) -> Finding {
    Finding {
        event_id: format!("event-{title}"),
        finding_key: format!("port.listen|tcp|0.0.0.0:{}", title.len()),
        kind: Kind::from("port.listen.new".to_string()),
        severity,
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
