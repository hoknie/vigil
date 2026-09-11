use serde_json::json;
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::budget::{CEILING_PERCENT, CPU, Meter};

use super::{rfc3339, uuid7};

const HOW_THE_SHARE_IS_TAKEN: &str = "the share of one core is the average of the last 8 readings of each collector divided by that collector's period, summed over the collectors";

pub fn store_damaged(lines: usize) -> Finding {
    let now = rfc3339::now();

    Finding {
        event_id: uuid7::mint(),
        finding_key: "agent.store|findings".to_string(),
        kind: Kind::Known(KnownKind::AgentStoreDamaged),
        severity: Severity::Medium,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: format!("{lines} line(s) of the local findings history could not be read"),
        subject: Subject {
            object: "store".into(),
            key: json!({ "part": "findings" }),
        },
        before: None,
        after: None,
        evidence: vec![Evidence {
            kind: "note".into(),
            value: format!(
                "{lines} unreadable line(s) were skipped when the journal was opened. The usual cause is power lost mid-write. What is lost is history; the watch continues"
            ),
        }],
        redacted: Vec::new(),
        rule: Some("store_health".into()),
        labels: Default::default(),
    }
}

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

#[cfg(test)]
mod tests {
    use vigil_model::Snapshot;

    use crate::types::Reading;

    use super::*;

    fn expensive() -> Meter {
        let mut meter = Meter::default();
        meter.record(&Reading {
            collector: "ports",
            at: "2026-09-10T12:00:00.000Z".into(),
            duration_ms: 600,
            every_seconds: 30,
            next_run_at: "2026-09-10T12:00:30.000Z".into(),
            skipped: 0,
            snapshot: Snapshot::new("ports", "2026-09-10T12:00:00.000Z"),
        });
        meter
    }

    #[test]
    fn the_finding_carries_the_schedule_line_that_fixes_it() {
        let finding = budget_exceeded(CPU, "This agent costs 2.00 % of one core", &expensive());

        let schedule = finding
            .evidence
            .iter()
            .find(|evidence| evidence.kind == "schedule")
            .expect("the way out is in the finding, not in a document");
        assert!(
            schedule.value.contains("schedule:\n  ports: 60"),
            "{schedule:?}"
        );
        assert!(
            finding
                .evidence
                .iter()
                .any(|evidence| evidence.kind == "cost"
                    && evidence.value.contains("ports")
                    && evidence.value.contains("every 30 s")),
            "the table names the collector, its share and its period"
        );
    }

    #[test]
    fn going_back_under_the_ceiling_closes_the_finding_rather_than_opening_a_second_one() {
        let opened = budget_exceeded(CPU, "over", &expensive());
        let closed = budget_recovered(CPU, "back under", "three checks in a row under the ceiling");

        assert_eq!(
            opened.finding_key, closed.finding_key,
            "the two are statements about one object"
        );
        assert_eq!(closed.kind, Kind::Known(KnownKind::AgentBudgetRecovered));
        assert_eq!(
            KnownKind::AgentBudgetRecovered.resolves(),
            Some(KnownKind::AgentBudgetExceeded)
        );
    }

    #[test]
    fn a_memory_ceiling_is_not_answered_with_a_reading_period() {
        let finding = budget_exceeded("resident", "This agent holds 70 MB", &expensive());

        assert!(
            !finding
                .evidence
                .iter()
                .any(|evidence| evidence.kind == "schedule"),
            "reading less often does not give memory back"
        );
    }
}
