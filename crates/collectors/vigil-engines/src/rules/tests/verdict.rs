use serde_json::Value;
use vigil_model::{Finding, Snapshot};
use vigil_rules::{RuleContext, diff};

use super::super::engine_rules;
use crate::fixture;
use crate::parsers::key;
use crate::types::{Engine, Report, Subject};

pub fn judged(before: &Snapshot, after: &Snapshot) -> Vec<Finding> {
    judged_under(&Report::default(), before, after)
}

pub fn judged_under(report: &Report, before: &Snapshot, after: &Snapshot) -> Vec<Finding> {
    let mut minted = 0;
    let mut mint = || {
        minted += 1;
        format!("event-{minted}")
    };
    let mut ctx = RuleContext {
        now: "2026-09-18T12:00:00.000Z".into(),
        mint_event_id: &mut mint,
    };

    engine_rules(report).judge(&diff(before, after), &mut ctx)
}

pub fn said(findings: &[Finding]) -> Vec<(String, String)> {
    let mut said: Vec<(String, String)> = findings
        .iter()
        .map(|finding| (finding.kind.to_string(), finding.finding_key.clone()))
        .collect();
    said.sort();
    said
}

pub fn with(
    reading: &Snapshot,
    engine: Engine,
    subject: Subject,
    id: &str,
    item: Value,
) -> Snapshot {
    let mut changed = reading.clone();
    changed.items.insert(key(engine, subject, id), item);
    changed
}

pub fn without(reading: &Snapshot, engine: Engine, subject: Subject, id: &str) -> Snapshot {
    let mut changed = reading.clone();
    changed.items.remove(&key(engine, subject, id));
    changed
}

pub fn altered(
    reading: &Snapshot,
    engine: Engine,
    subject: Subject,
    id: &str,
    field: &str,
    value: Value,
) -> Snapshot {
    let mut item = fixture::row(reading, engine, subject, id);
    item[field] = value;
    with(reading, engine, subject, id, item)
}

pub fn silenced_everything() -> Report {
    Report {
        images: false,
        volumes: false,
        networks: false,
        projects: false,
        pods: false,
        secrets: false,
    }
}
