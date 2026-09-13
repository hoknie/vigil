use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct UnitCommandChanged;

impl Rule for UnitCommandChanged {
    fn name(&self) -> &'static str {
        "unit_command_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Unit) {
            return None;
        }
        let was = PersistenceView::new(key, before);

        if was.commands() == view.commands() && was.run_as() == view.run_as() {
            return None;
        }
        if was.readable() && !view.readable() {
            return None;
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::FileChanged,
                severity: match view.runs_from_writable_path() {
                    true => Severity::Critical,
                    false => Severity::High,
                },
                rule: self.name(),
                key,
                object: "unit",
                title: format!(
                    "systemd unit {} now runs {} as {}",
                    view.name(),
                    view.commands(),
                    view.run_as()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    Evidence {
                        kind: "path".into(),
                        value: view.path().to_string(),
                    },
                    Evidence {
                        kind: "cmdline".into(),
                        value: format!("was: {} as {}", was.commands(), was.run_as()),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        UnitCommandChanged.apply(change, &mut ctx)
    }

    #[test]
    fn an_exec_start_pointed_somewhere_else_is_reported_with_both_halves() {
        let finding = apply(&Change::Changed {
            key: "unit|nginx.service".into(),
            before: fixture::unit("nginx.service", "/usr/sbin/nginx", "root"),
            after: fixture::unit("nginx.service", "/tmp/.x/nginx", "root"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.kind.as_str(), "file.changed");
        assert!(finding.title.contains("/tmp/.x/nginx"), "{}", finding.title);
        assert!(
            finding
                .evidence
                .iter()
                .any(|e| e.value.contains("/usr/sbin/nginx")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_unit_that_now_runs_as_somebody_else_is_the_same_event() {
        let finding = apply(&Change::Changed {
            key: "unit|worker.service".into(),
            before: fixture::unit("worker.service", "/usr/bin/worker", "worker"),
            after: fixture::unit("worker.service", "/usr/bin/worker", "root"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("as root"), "{}", finding.title);
    }

    #[test]
    fn a_description_that_changed_is_not_an_event() {
        let before = fixture::unit("nginx.service", "/usr/sbin/nginx", "root");
        let mut after = before.clone();
        after["description"] = serde_json::json!("A web server, corrected");

        assert!(
            apply(&Change::Changed {
                key: "unit|nginx.service".into(),
                before,
                after,
            })
            .is_none()
        );
    }

    #[test]
    fn a_unit_the_agent_stopped_being_able_to_read_is_not_a_unit_that_stopped_running_anything() {
        let before = fixture::unit("secret.service", "/usr/bin/thing", "root");
        let mut after = before.clone();
        after["readable"] = serde_json::json!(false);
        after["commands"] = serde_json::json!([]);

        assert!(
            apply(&Change::Changed {
                key: "unit|secret.service".into(),
                before,
                after,
            })
            .is_none()
        );
    }
}
