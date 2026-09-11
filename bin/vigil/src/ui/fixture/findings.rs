use serde_json::json;
use vigil_model::{Finding, Kind, Severity, State, Subject};

pub fn finding(title: &str, severity: Severity) -> Finding {
    let mut mark: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in title.as_bytes() {
        mark = (mark ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3);
    }

    Finding {
        event_id: format!("0199a1b2-c3d4-7e5f-8a9b-{:012x}", mark & 0xffff_ffff_ffff),
        finding_key: "port.listen|tcp|0.0.0.0:4444".into(),
        kind: Kind::from("port.listen.new".to_string()),
        severity,
        state: State::Open,
        observed_at: "2026-09-09T09:00:00.000Z".into(),
        first_seen_at: "2026-09-09T09:00:00.000Z".into(),
        occurrences: 1,
        title: title.to_string(),
        subject: Subject {
            object: "socket".into(),
            key: json!({"protocol": "tcp", "address": "0.0.0.0", "port": 4444}),
        },
        before: None,
        after: Some(json!({
            "protocol": "tcp", "address": "0.0.0.0", "port": 4444, "user": "www-data",
            "process": {"exe": "/tmp/.x/nc", "exe_deleted": true},
        })),
        evidence: Vec::new(),
        redacted: Vec::new(),
        rule: Some("new_listening_port".into()),
        labels: Default::default(),
    }
}
