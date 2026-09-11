use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::process_view::ProcessView;
use crate::RuleContext;

pub struct ProcessFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

const FINDING_KEY_PREFIX: &str = "process";

pub fn build(spec: ProcessFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = ProcessView::new(value);
            json!({ "executable": view.executable(), "user": view.user() })
        }
        None => json!({ "key": spec.key }),
    };

    let mut redacted = Vec::new();
    for (pointer, value) in [("/before", &spec.before), ("/after", &spec.after)] {
        if let Some(value) = value
            && ProcessView::new(value).command_line_redacted()
        {
            redacted.push(format!("{pointer}/cmdline"));
        }
    }

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: format!("{FINDING_KEY_PREFIX}|{}", spec.key),
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Subject {
            object: "process".into(),
            key: subject,
        },
        before: spec.before,
        after: spec.after,
        evidence: spec.evidence,
        redacted,
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn program_evidence(view: &ProcessView<'_>) -> Vec<Evidence> {
    let mut evidence = vec![Evidence {
        kind: "path".into(),
        value: match view.executable_deleted() {
            true => format!(
                "{} (deleted from disk while still running)",
                view.executable()
            ),
            false => view.executable().to_string(),
        },
    }];

    match view.command_line() {
        Some(command) => evidence.push(Evidence {
            kind: "cmdline".into(),
            value: command.to_string(),
        }),
        None if view.command_line_varies() => evidence.push(Evidence {
            kind: "note".into(),
            value: "the running copies have different command lines".into(),
        }),
        None => {}
    }

    if !view.parents().is_empty() {
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!("started by {}", view.parents().join(", ")),
        });
    }

    evidence
}
