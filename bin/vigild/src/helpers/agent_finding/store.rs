use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::helpers::{rfc3339, uuid7};

pub fn store_damaged(part: &str, what: &str, lines: usize) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.store|{part}"),
        kind: Kind::Known(KnownKind::AgentStoreDamaged),
        severity: Severity::Medium,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!("{lines} line(s) of {what} could not be read"),
        subject: Subject {
            object: "store".into(),
            key: json!({ "part": part }),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "note".into(),
            value: format!(
                "{lines} unreadable line(s) were skipped when {what} was opened. The usual cause is power lost mid-write. What is lost is history; the watch continues"
            ),
        }],
        redacted: Vec::new(),
        rule: Some("store_health".into()),
        labels: Default::default(),
    }
}
