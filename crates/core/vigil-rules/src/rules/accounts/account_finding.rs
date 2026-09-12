use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::sudoer_view::SudoerView;
use crate::RuleContext;

pub struct AccountFinding<'a> {
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

const FINDING_KEY_PREFIX: &str = "user";

pub fn build(spec: AccountFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let mut redacted = Vec::new();
    for (pointer, value) in [("/before", &spec.before), ("/after", &spec.after)] {
        if let Some(value) = value
            && SudoerView::new(value).spec_redacted()
        {
            redacted.push(format!("{pointer}/rules/*/spec"));
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
            key: json!({ "key": spec.key }),
        },
        before: spec.before,
        after: spec.after,
        evidence: spec.evidence,
        redacted,
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn note(text: impl Into<String>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: text.into(),
    }
}
