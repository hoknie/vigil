use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::persistence_view::{Family, PersistenceView};
use crate::RuleContext;

pub struct PersistenceFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

const FINDING_KEY_PREFIX: &str = "persistence";

pub fn build(spec: PersistenceFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = PersistenceView::new(spec.key, value);
            match view.family() {
                Some(Family::Cron) => json!({
                    "source": view.source(),
                    "user": view.user(),
                    "command": view.command(),
                }),
                Some(Family::Script) | Some(Family::Preload) => json!({ "path": view.path() }),
                _ => json!({ "name": view.name(), "path": view.path() }),
            }
        }
        None => json!({ "key": spec.key }),
    };

    let mut redacted = Vec::new();
    for (pointer, value) in [("/before", &spec.before), ("/after", &spec.after)] {
        let Some(value) = value else { continue };
        let view = PersistenceView::new(spec.key, value);
        if view.commands_redacted() {
            redacted.push(format!("{pointer}/commands"));
        }
        if view.command_redacted() {
            redacted.push(format!("{pointer}/command"));
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
            object: spec.object.into(),
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

pub fn origin_evidence(view: &PersistenceView<'_>) -> Vec<Evidence> {
    let mut evidence = vec![Evidence {
        kind: "path".into(),
        value: view.path().to_string(),
    }];

    if !view.readable() {
        evidence.push(Evidence {
            kind: "note".into(),
            value: "the file exists and is not readable; what it starts is unknown".into(),
        });
        return evidence;
    }

    let commands = view.commands();
    if !commands.is_empty() {
        evidence.push(Evidence {
            kind: "cmdline".into(),
            value: commands,
        });
    }
    if let Some(description) = view.description() {
        evidence.push(Evidence {
            kind: "note".into(),
            value: description.to_string(),
        });
    }

    evidence
}
