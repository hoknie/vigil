use serde_json::json;
use vigil_model::{CollectorStatus, Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::helpers::{rfc3339, uuid7};

pub fn collector_degraded(collector: &str, detail: &str, fatal: bool) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.collector|{collector}"),
        kind: Kind::Known(KnownKind::AgentCollectorDegraded),
        severity: match fatal {
            true => Severity::High,
            false => Severity::Medium,
        },
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: match fatal {
            true => format!("Collector {collector}: unavailable on this host"),
            false => {
                format!("Collector {collector}: degraded, part of what it watches is unreadable")
            }
        },
        subject: Subject {
            object: "collector".into(),
            key: json!({ "name": collector }),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "note".into(),
            value: detail.to_string(),
        }],
        redacted: Vec::new(),
        rule: Some("collector_health".into()),
        labels: Default::default(),
    }
}

pub fn collector_recovered(collector: &str, was: &str) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.collector|{collector}"),
        kind: Kind::Known(KnownKind::AgentCollectorRecovered),
        severity: Severity::Info,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!("Collector {collector}: reading again"),
        subject: Subject {
            object: "collector".into(),
            key: json!({ "name": collector }),
        },
        before: None,
        after: None,
        evidence: vec![
            Evidence {
                kind: "note".into(),
                value: format!(
                    "{collector} is reading everything it watches again. Whatever happened while it could not is missing from the history of this host; the readings from here on are complete"
                ),
            },
            Evidence {
                kind: "was".into(),
                value: was.to_string(),
            },
        ],
        redacted: Vec::new(),
        rule: Some("collector_health".into()),
        labels: Default::default(),
    }
}

pub fn collector_failing(collector: &str, status: &CollectorStatus) -> Finding {
    let now = rfc3339::now();
    let mut evidence = vec![Evidence {
        kind: "note".into(),
        value: match status.readings {
            0 => format!(
                "{collector} says it can run on this host and has not completed a reading on it yet, so the failure is in the reading itself rather than in a source that is not here"
            ),
            readings => format!(
                "{collector} read this host {readings} time(s) and the reading it has just tried failed: something here changed a moment ago, and what it watches is unwatched until it reads again"
            ),
        },
    }];
    evidence.push(Evidence {
        kind: "error".into(),
        value: status
            .last_error
            .clone()
            .unwrap_or_else(|| "the reading failed and named no cause".to_string()),
    });
    evidence.push(Evidence {
        kind: "cost".into(),
        value: format!(
            "{} failed reading(s), {} that went through, {} slot(s) missed{}",
            status.failures,
            status.readings,
            status.skipped,
            match &status.last_run_at {
                Some(at) => format!(", last tried at {at}"),
                None => String::new(),
            }
        ),
    });

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.collector|{collector}"),
        kind: Kind::Known(KnownKind::AgentCollectorDegraded),
        severity: Severity::High,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!("Collector {collector}: the reading failed"),
        subject: Subject {
            object: "collector".into(),
            key: json!({ "name": collector }),
        },
        before: None,
        after: None,
        evidence,
        redacted: Vec::new(),
        rule: Some("collector_health".into()),
        labels: Default::default(),
    }
}
