use vigil_model::{Change, Finding, KnownKind, Severity};

use super::launch_finding::{LaunchFinding, build, launch_evidence};
use super::launch_view::LaunchView;
use crate::{Rule, RuleContext};

pub struct LaunchFromWritablePath;

impl Rule for LaunchFromWritablePath {
    fn name(&self) -> &'static str {
        "launch_from_writable_path"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = LaunchView::new(after);
        if !view.is_launch() || !view.runs_from_writable_path() {
            return None;
        }

        let gone = match view.on_disk() {
            true => "",
            false => ", file already gone",
        };

        Some(build(
            LaunchFinding {
                kind: KnownKind::ExecFromWritablePath,
                severity: Severity::High,
                rule: self.name(),
                key,
                title: format!(
                    "{} ran {} from a directory anybody can write to{gone}",
                    view.user(),
                    view.executable()
                ),
                after: after.clone(),
                evidence: launch_evidence(&view),
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        LaunchFromWritablePath.apply(change, &mut ctx)
    }

    #[test]
    fn a_program_run_out_of_dev_shm_is_reported_loudly() {
        let finding = apply(&Change::Added {
            key: "run|alice|/dev/shm/payload".into(),
            after: fixture::launch("alice", 1000, "/dev/shm/payload"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.kind.as_str(), "exec.from_writable_path");
        assert_eq!(finding.finding_key, "run|alice|/dev/shm/payload");
    }

    #[test]
    fn a_dropper_that_deleted_itself_is_one_finding_and_says_both_things() {
        let finding = apply(&Change::Added {
            key: "run|alice|/tmp/.x/dropper".into(),
            after: fixture::launch_of_a_missing_program("alice", 1000, "/tmp/.x/dropper"),
        })
        .expect("fires");

        assert!(finding.title.contains("already gone"), "{}", finding.title);
    }

    #[test]
    fn a_program_where_programs_live_is_not_this_rule() {
        let change = Change::Added {
            key: "run|alice|/usr/bin/nmap".into(),
            after: fixture::launch("alice", 1000, "/usr/bin/nmap"),
        };

        assert!(apply(&change).is_none());
    }
}
