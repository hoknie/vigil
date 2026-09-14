use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::budget::{CEILING_PERCENT, CPU, Meter};
use crate::helpers::{rfc3339, uuid7};

const HOW_THE_SHARE_IS_TAKEN: &str = "the share of one core is the average of the last 8 readings of each collector divided by that collector's period, summed over the collectors";

pub fn budget_exceeded(part: &str, title: &str, meter: &Meter) -> Finding {
    let mut evidence = vec![Evidence {
        kind: "note".into(),
        value: format!("{HOW_THE_SHARE_IS_TAKEN}. The ceiling is {CEILING_PERCENT} % of one core"),
    }];
    for cost in meter.costs() {
        evidence.push(Evidence {
            kind: "cost".into(),
            value: cost,
        });
    }
    if part == CPU {
        evidence.push(Evidence {
            kind: "schedule".into(),
            value: format!(
                "reading less often is a decision for a person to take, so this agent does not take it. To take it, put this in the configuration:\n{}",
                meter.schedule_line()
            ),
        });
    }

    budget_finding(part, title, KnownKind::AgentBudgetExceeded, evidence)
}

pub fn budget_recovered(part: &str, title: &str, note: &str) -> Finding {
    budget_finding(
        part,
        title,
        KnownKind::AgentBudgetRecovered,
        vec![Evidence {
            kind: "note".into(),
            value: note.to_string(),
        }],
    )
}

fn budget_finding(part: &str, title: &str, kind: KnownKind, evidence: Vec<Evidence>) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: format!("agent.budget|{part}"),
        kind: Kind::Known(kind),
        severity: Severity::Medium,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: title.to_string(),
        subject: Subject {
            object: "agent".into(),
            key: json!({ "part": part }),
        },
        before: None,
        after: None,
        evidence,
        redacted: Vec::new(),
        rule: Some("agent_budget".into()),
        labels: Default::default(),
    }
}
