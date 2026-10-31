use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::helpers::{rfc3339, uuid7};

pub fn baseline_ready(collector: &str, items: usize) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.baseline|{collector}"),
        kind: Kind::Known(KnownKind::AgentBaselineReady),
        severity: Severity::Info,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!(
            "Collector {collector}: first reading kept as the baseline, changes are reported from now on"
        ),
        subject: Subject {
            object: "collector".into(),
            key: json!({ "collector": collector }),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "items".into(),
            value: format!("{items} item(s) in the baseline"),
        }],
        redacted: Vec::new(),
        rule: Some("baseline_ready".into()),
        labels: Default::default(),
    }
}
