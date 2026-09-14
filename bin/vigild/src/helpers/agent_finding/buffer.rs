use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};
use vigil_store::Held;

use crate::helpers::{rfc3339, uuid7};

const KILOBYTE: u64 = 1_024;

pub fn buffer_dropping(reporter: &str, dropped: u64, held: &Held) -> Finding {
    let now = rfc3339::now();
    let mut evidence = vec![Evidence {
        kind: "note".into(),
        value: format!(
            "{} finding(s) and {} kB are held for {reporter}, which is the ceiling ({} finding(s), {} kB). What arrives now displaces the oldest still waiting, and the oldest is what this receiver has never seen",
            held.records.held,
            held.bytes.held / KILOBYTE,
            held.records.ceiling,
            held.bytes.ceiling / KILOBYTE,
        ),
    }];
    if let Some(oldest_at) = &held.oldest_at {
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!("the oldest finding still waiting was seen at {oldest_at}"),
        });
    }

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.buffer|{reporter}"),
        kind: Kind::Known(KnownKind::AgentBufferDropping),
        severity: Severity::High,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!(
            "The buffer for {reporter} is full: {dropped} finding(s) dropped, oldest first"
        ),
        subject: Subject {
            object: "buffer".into(),
            key: json!({ "reporter": reporter }),
        },
        before: None,
        after: None,
        evidence,
        redacted: Vec::new(),
        rule: Some("outgoing_buffer".into()),
        labels: Default::default(),
    }
}

pub fn buffer_drained(reporter: &str, dropped: u64) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.buffer|{reporter}"),
        kind: Kind::Known(KnownKind::AgentBufferDrained),
        severity: Severity::Info,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!("The buffer for {reporter} is empty again"),
        subject: Subject {
            object: "buffer".into(),
            key: json!({ "reporter": reporter }),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "note".into(),
            value: format!(
                "everything that was waiting has been accepted. The {dropped} finding(s) dropped while the buffer was full are in the local history and reached this receiver never"
            ),
        }],
        redacted: Vec::new(),
        rule: Some("outgoing_buffer".into()),
        labels: Default::default(),
    }
}
