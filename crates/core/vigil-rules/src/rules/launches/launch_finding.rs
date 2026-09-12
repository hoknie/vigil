use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::launch_view::LaunchView;
use crate::RuleContext;

pub struct LaunchFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub title: String,
    pub after: Value,
    pub evidence: Vec<Evidence>,
}

pub fn build(spec: LaunchFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let view = LaunchView::new(&spec.after);

    let mut redacted = Vec::new();
    if view.arguments_redacted() {
        redacted.push("/after/arguments".to_string());
    }

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: spec.key.to_string(),
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Subject {
            object: "launch".into(),
            key: json!({ "user": view.user(), "executable": view.executable() }),
        },
        before: None,
        after: Some(spec.after.clone()),
        evidence: spec.evidence,
        redacted,
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn launch_evidence(view: &LaunchView<'_>) -> Vec<Evidence> {
    let mut evidence = vec![Evidence {
        kind: "path".into(),
        value: match view.on_disk() {
            true => view.executable().to_string(),
            false => format!("{} (not on disk when read)", view.executable()),
        },
    }];

    if view.executable_lossy() {
        evidence.push(Evidence {
            kind: "note".into(),
            value: "the path holds bytes that are not text; shown above as an approximation".into(),
        });
    }

    evidence.push(Evidence {
        kind: "note".into(),
        value: format!(
            "started in the login session of {} (login uid {})",
            view.user(),
            view.auid()
        ),
    });

    if let Some(arguments) = view.arguments() {
        evidence.push(Evidence {
            kind: "cmdline".into(),
            value: arguments.to_string(),
        });
    }

    if let Some(id) = view.audit_id() {
        let serial = id.rsplit_once(':').map(|(_, serial)| serial).unwrap_or(id);
        evidence.push(Evidence {
            kind: "note".into(),
            value: format!("auditd record {id}; read it with `ausearch -a {serial}`"),
        });
    }

    evidence
}
