use vigil_model::{Change, Finding, KnownKind, Severity};

use super::process_finding::{ProcessFinding, build, program_evidence};
use super::process_view::ProcessView;
use crate::{Rule, RuleContext};

pub struct ProcessBinaryDeleted;

impl Rule for ProcessBinaryDeleted {
    fn name(&self) -> &'static str {
        "process_binary_deleted"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let view = ProcessView::new(after);
        if !view.is_program() || !view.executable_deleted() || view.runs_from_writable_path() {
            return None;
        }
        if view.is_shell_under_a_service() {
            return None;
        }
        if before.is_some_and(|value| ProcessView::new(value).executable_deleted()) {
            return None;
        }

        Some(build(
            ProcessFinding {
                kind: KnownKind::ProcessBinaryDeleted,
                severity: Severity::High,
                rule: self.name(),
                key,
                title: format!(
                    "{} is running as {} from a deleted file",
                    view.executable(),
                    view.user()
                ),
                before: before.cloned(),
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
        ProcessBinaryDeleted.apply(change, &mut ctx)
    }

    #[test]
    fn a_running_program_whose_file_went_away_is_reported_once() {
        let before = fixture::program("/usr/sbin/nginx", "root", 0, &[]);
        let mut after = before.clone();
        after["exe_deleted"] = serde_json::json!(true);

        let finding = apply(&Change::Changed {
            key: "exec|/usr/sbin/nginx|root".into(),
            before: before.clone(),
            after: after.clone(),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.kind.as_str(), "process.binary_deleted");

        let mut later = after.clone();
        later["cmdline"] = serde_json::json!("nginx -g 'daemon off;'");
        assert!(
            apply(&Change::Changed {
                key: "exec|/usr/sbin/nginx|root".into(),
                before: after,
                after: later,
            })
            .is_none()
        );
    }

    #[test]
    fn a_deleted_binary_in_a_writable_directory_is_the_louder_rules_to_report() {
        let mut value = fixture::program("/tmp/.x/nc", "www-data", 33, &[]);
        value["exe_deleted"] = serde_json::json!(true);

        assert!(
            apply(&Change::Added {
                key: "exec|/tmp/.x/nc|www-data".into(),
                after: value,
            })
            .is_none()
        );
    }
}
