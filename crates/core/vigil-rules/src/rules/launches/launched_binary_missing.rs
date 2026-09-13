use vigil_model::{Change, Finding, KnownKind, Severity};

use super::launch_finding::{LaunchFinding, build, launch_evidence};
use super::launch_view::LaunchView;
use crate::{Rule, RuleContext};

pub struct LaunchedBinaryMissing;

impl Rule for LaunchedBinaryMissing {
    fn name(&self) -> &'static str {
        "launched_binary_missing"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = LaunchView::new(after);
        if !view.is_launch() || view.on_disk() {
            return None;
        }
        if view.runs_from_writable_path() {
            return None;
        }

        Some(build(
            LaunchFinding {
                kind: KnownKind::ProcessBinaryDeleted,
                severity: Severity::High,
                rule: self.name(),
                key,
                title: format!(
                    "{} ran {}, not on disk when the launch was read",
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
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        LaunchedBinaryMissing.apply(change, &mut ctx)
    }

    #[test]
    fn a_program_that_had_already_been_unlinked_is_reported_in_the_words_the_vocabulary_has() {
        let finding = apply(&Change::Added {
            key: "run|alice|/opt/build/tool".into(),
            after: fixture::launch_of_a_missing_program("alice", 1000, "/opt/build/tool"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.kind.as_str(), "process.binary_deleted");
        assert_eq!(finding.finding_key, "run|alice|/opt/build/tool");
    }

    #[test]
    fn a_program_that_is_still_there_is_not_this_rule() {
        let change = Change::Added {
            key: "run|alice|/usr/bin/nmap".into(),
            after: fixture::launch("alice", 1000, "/usr/bin/nmap"),
        };

        assert!(apply(&change).is_none());
    }
}
