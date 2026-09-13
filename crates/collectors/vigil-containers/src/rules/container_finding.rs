use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::types::{ContainerView, Family};
use vigil_rules::RuleContext;

pub struct ContainerFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub finding_key: String,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

pub fn build(spec: ContainerFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let describing = spec.after.as_ref().or(spec.before.as_ref());
    let subject = match describing {
        Some(value) => {
            let view = ContainerView::new(spec.key, value);
            match view.family() {
                Some(Family::Socket) => json!({ "path": view.path() }),
                _ => json!({
                    "container": view.short(),
                    "runtime": view.runtime(),
                    "exe": view.executable(),
                }),
            }
        }
        None => json!({ "key": spec.key }),
    };

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: spec.finding_key,
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
        redacted: Vec::new(),
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn finding_key(what: &str, view: &ContainerView<'_>) -> String {
    format!("container|{what}|{}", view.identity())
}

pub fn running(view: &ContainerView<'_>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: format!(
            "{} container {}, running {}, holds {} path(s) of this host",
            view.runtime(),
            view.short(),
            view.executable()
                .unwrap_or("a program this agent may not read"),
            view.host_paths().len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    #[test]
    fn a_finding_about_a_container_is_keyed_by_what_it_runs_and_not_by_the_identifier_of_the_day() {
        let container = fixture::container("/usr/sbin/nginx", "000001ffffffffff", &["/etc"]);
        let view = ContainerView::new("container|3ab1c0f2d4e5", &container);

        assert_eq!(
            finding_key("privileged", &view),
            "container|privileged|/usr/sbin/nginx"
        );
        assert!(running(&view).value.contains("3ab1c0f2d4e5"));
    }
}
