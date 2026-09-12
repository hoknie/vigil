use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use super::socket_view::SocketView;
use crate::RuleContext;

pub struct SocketFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

const FINDING_KEY_PREFIX: &str = "port.listen";

pub fn build(spec: SocketFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = SocketView::new(value);
            match view.path() {
                Some(path) => json!({ "protocol": view.protocol(), "path": path }),
                None => json!({
                    "protocol": view.protocol(),
                    "address": view.address(),
                    "port": view.port(),
                }),
            }
        }
        None => json!({ "key": spec.key }),
    };

    let mut redacted = Vec::new();
    for (pointer, value) in [("/before", &spec.before), ("/after", &spec.after)] {
        if let Some(value) = value
            && SocketView::new(value).command_line_redacted()
        {
            redacted.push(format!("{pointer}/process/cmdline"));
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
            object: "socket".into(),
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

pub fn owner_evidence(view: &SocketView<'_>) -> Vec<Evidence> {
    let mut evidence = Vec::new();

    if !view.owner_resolved() {
        evidence.push(Evidence {
            kind: "note".into(),
            value: "owner not resolved: cannot read /proc/<pid>/fd (needs CAP_SYS_PTRACE and CAP_DAC_READ_SEARCH)".into(),
        });
        return evidence;
    }

    if let Some(executable) = view.executable() {
        evidence.push(Evidence {
            kind: "path".into(),
            value: match view.executable_deleted() {
                true => format!("{executable} (deleted from disk while still running)"),
                false => executable.to_string(),
            },
        });
    }
    if let Some(command_line) = view.command_line() {
        evidence.push(Evidence {
            kind: "cmdline".into(),
            value: command_line.to_string(),
        });
    }

    evidence
}
