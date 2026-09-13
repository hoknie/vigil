use vigil_model::{Change, Finding, KnownKind, Severity};

use super::process_finding::{ProcessFinding, build, program_evidence};
use super::process_view::ProcessView;
use crate::{Rule, RuleContext};

pub struct NewRootProcess;

impl Rule for NewRootProcess {
    fn name(&self) -> &'static str {
        "new_root_process"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = ProcessView::new(after);
        if !view.is_program() || !view.is_root() {
            return None;
        }
        if view.runs_from_writable_path()
            || view.executable_deleted()
            || view.is_shell_under_a_service()
        {
            return None;
        }

        Some(build(
            ProcessFinding {
                kind: KnownKind::ProcessRootNew,
                severity: Severity::Low,
                rule: self.name(),
                key,
                title: format!("{} ran as root for the first time", view.executable()),
                before: None,
                after: Some(after.clone()),
                evidence: program_evidence(&view),
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
        NewRootProcess.apply(change, &mut ctx)
    }

    #[test]
    fn a_new_root_program_is_reported_quietly_and_under_a_key_a_person_can_suppress() {
        let finding = apply(&Change::Added {
            key: "exec|/usr/bin/tcpdump|root".into(),
            after: fixture::program("/usr/bin/tcpdump", "root", 0, &[]),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Low);
        assert_eq!(finding.kind.as_str(), "process.root.new");
        assert_eq!(finding.finding_key, "process|exec|/usr/bin/tcpdump|root");
    }

    #[test]
    fn a_program_running_as_somebody_else_is_not_this_rule() {
        let change = Change::Added {
            key: "exec|/usr/sbin/nginx|www-data".into(),
            after: fixture::program("/usr/sbin/nginx", "www-data", 33, &[]),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn the_louder_rules_keep_it_quiet() {
        let mut deleted = fixture::program("/usr/sbin/nginx", "root", 0, &[]);
        deleted["exe_deleted"] = serde_json::json!(true);

        assert!(
            apply(&Change::Added {
                key: "exec|/usr/sbin/nginx|root".into(),
                after: deleted,
            })
            .is_none()
        );
        assert!(
            apply(&Change::Added {
                key: "exec|/tmp/.x/nc|root".into(),
                after: fixture::program("/tmp/.x/nc", "root", 0, &[]),
            })
            .is_none()
        );
    }
}
