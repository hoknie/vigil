use vigil_model::{Change, Finding, KnownKind, Severity};

use super::launch_finding::{LaunchFinding, build, launch_evidence};
use super::launch_view::LaunchView;
use crate::{Rule, RuleContext};

pub struct FirstLaunchForUser;

impl Rule for FirstLaunchForUser {
    fn name(&self) -> &'static str {
        "first_launch_for_user"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = LaunchView::new(after);
        if !view.is_launch() {
            return None;
        }
        if view.runs_from_writable_path() || !view.on_disk() {
            return None;
        }

        Some(build(
            LaunchFinding {
                kind: KnownKind::ExecFirstSeenForUser,
                severity: Severity::Low,
                rule: self.name(),
                key,
                title: format!(
                    "{} ran {} for the first time",
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
        FirstLaunchForUser.apply(change, &mut ctx)
    }

    #[test]
    fn a_first_launch_is_reported_quietly_under_the_key_that_is_on_the_screen() {
        let finding = apply(&Change::Added {
            key: "run|alice|/usr/bin/nmap".into(),
            after: fixture::launch("alice", 1000, "/usr/bin/nmap"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Low);
        assert_eq!(finding.kind.as_str(), "exec.first_seen_for_user");
        assert_eq!(finding.finding_key, "run|alice|/usr/bin/nmap");
        assert_eq!(finding.subject.key["user"], "alice");
    }

    #[test]
    fn the_louder_rules_keep_it_quiet() {
        assert!(
            apply(&Change::Added {
                key: "run|alice|/tmp/.x/dropper".into(),
                after: fixture::launch("alice", 1000, "/tmp/.x/dropper"),
            })
            .is_none()
        );
        assert!(
            apply(&Change::Added {
                key: "run|alice|/usr/bin/gone".into(),
                after: fixture::launch_of_a_missing_program("alice", 1000, "/usr/bin/gone"),
            })
            .is_none()
        );
    }

    #[test]
    fn a_launch_of_a_program_a_service_account_never_logged_in_to_run_is_not_this_family() {
        let change = Change::Added {
            key: "exec|/usr/sbin/nginx|www-data".into(),
            after: fixture::program("/usr/sbin/nginx", "www-data", 33, &[]),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn the_evidence_says_where_to_look_the_launch_up_in_the_hosts_own_log() {
        let finding = apply(&Change::Added {
            key: "run|alice|/usr/bin/nmap".into(),
            after: fixture::launch("alice", 1000, "/usr/bin/nmap"),
        })
        .expect("fires");

        assert!(
            finding
                .evidence
                .iter()
                .any(|fact| fact.value.contains("ausearch -a 3421")),
            "{:?}",
            finding.evidence
        );
    }
}
