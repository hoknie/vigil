use vigil_model::{Change, Finding, KnownKind, Severity};

use super::process_finding::{ProcessFinding, build, program_evidence};
use crate::types::ProcessView;
use vigil_rules::{Rule, RuleContext};

pub struct ProcessFromWritablePath;

impl Rule for ProcessFromWritablePath {
    fn name(&self) -> &'static str {
        "process_from_writable_path"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let view = ProcessView::new(after);
        if !view.is_program() || !view.runs_from_writable_path() {
            return None;
        }
        if view.is_shell_under_a_service() {
            return None;
        }
        let file_just_deleted = match before {
            None => false,
            Some(before) => {
                if ProcessView::new(before).executable_deleted() || !view.executable_deleted() {
                    return None;
                }
                true
            }
        };

        Some(build(
            ProcessFinding {
                kind: KnownKind::ProcessFromWritablePath,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                title: format!(
                    "{} is running from a directory anybody can write to, as {}{}",
                    view.executable(),
                    view.user(),
                    match file_just_deleted {
                        true => ", and its file has just been deleted",
                        false => "",
                    }
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
        ProcessFromWritablePath.apply(change, &mut ctx)
    }

    #[test]
    fn a_binary_in_dev_shm_is_reported_at_the_top_of_the_scale() {
        let finding = apply(&Change::Added {
            key: "exec|/dev/shm/payload|www-data".into(),
            after: fixture::program("/dev/shm/payload", "www-data", 33, &[]),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.kind.as_str(), "process.from_writable_path");
        assert_eq!(
            finding.finding_key,
            "process|exec|/dev/shm/payload|www-data"
        );
    }

    #[test]
    fn a_program_where_programs_live_is_not_this_rule() {
        let change = Change::Added {
            key: "exec|/usr/sbin/nginx|root".into(),
            after: fixture::program("/usr/sbin/nginx", "root", 0, &[]),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_program_that_stopped_running_is_not_a_program_that_started() {
        let change = Change::Removed {
            key: "exec|/tmp/.x/nc|www-data".into(),
            before: fixture::program("/tmp/.x/nc", "www-data", 33, &[]),
        };

        assert!(apply(&change).is_none());
    }
    #[test]
    fn a_program_from_a_writable_directory_that_deletes_its_own_file_is_reported_at_the_top() {
        let finding = apply(&Change::Changed {
            key: "exec|/tmp/.x/nc|www-data".into(),
            before: fixture::program("/tmp/.x/nc", "www-data", 33, &[]),
            after: fixture::program_with_deleted_binary("/tmp/.x/nc", "www-data", 33),
        })
        .expect("a program that ran from /tmp at the baseline and then wiped its file is news");

        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.kind.as_str(), "process.from_writable_path");
        assert!(
            finding
                .title
                .ends_with("and its file has just been deleted"),
            "{}",
            finding.title
        );
        assert!(finding.before.is_some());
    }

    #[test]
    fn a_program_from_a_writable_directory_that_only_changed_its_arguments_says_nothing() {
        let before = fixture::program("/tmp/.x/nc", "www-data", 33, &[]);
        let mut after = before.clone();
        after["cmdline"] = serde_json::json!("/tmp/.x/nc -l 4444");

        assert!(
            apply(&Change::Changed {
                key: "exec|/tmp/.x/nc|www-data".into(),
                before,
                after,
            })
            .is_none()
        );
    }

    #[test]
    fn a_file_already_deleted_at_the_reading_before_is_not_reported_again() {
        assert!(
            apply(&Change::Changed {
                key: "exec|/tmp/.x/nc|www-data".into(),
                before: fixture::program_with_deleted_binary("/tmp/.x/nc", "www-data", 33),
                after: fixture::program_with_deleted_binary("/tmp/.x/nc", "www-data", 34),
            })
            .is_none()
        );
    }
}
