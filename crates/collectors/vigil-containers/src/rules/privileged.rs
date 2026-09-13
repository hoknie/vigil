use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::container_finding::{ContainerFinding, build, finding_key, running};
use crate::types::{ContainerView, Family};
use vigil_rules::{Rule, RuleContext};

pub const SYS_ADMIN: u64 = 1 << 21;

pub struct ContainerPrivileged;

impl Rule for ContainerPrivileged {
    fn name(&self) -> &'static str {
        "container_privileged"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = ContainerView::new(key, after);
        if !now.is(Family::Container) || !raised(&now) {
            return None;
        }
        if let Some(before) = before
            && raised(&ContainerView::new(key, before))
        {
            return None;
        }

        Some(build(
            ContainerFinding {
                kind: KnownKind::ContainerPrivileged,
                severity: Severity::High,
                rule: self.name(),
                key,
                finding_key: finding_key("privileged", &now),
                object: "container",
                title: format!(
                    "The {} container running {} holds CAP_SYS_ADMIN, which is the whole host: it can mount filesystems, load what the kernel will take and leave the container at will",
                    now.runtime(),
                    now.executable()
                        .unwrap_or("a program this agent may not read")
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    running(&now),
                    Evidence {
                        kind: "note".into(),
                        value: format!(
                            "effective capabilities {}; a container started without --privileged and without --cap-add SYS_ADMIN does not have this one",
                            now.capabilities()
                                .map(|mask| format!("{mask:016x}"))
                                .unwrap_or_else(|| "unknown".to_string())
                        ),
                    },
                ],
            },
            ctx,
        ))
    }
}

fn raised(view: &ContainerView<'_>) -> bool {
    view.capabilities()
        .is_some_and(|capabilities| capabilities & SYS_ADMIN != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    const ORDINARY: &str = "00000000a80425fb";

    const EVERYTHING: &str = "000001ffffffffff";

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ContainerPrivileged.apply(change, &mut ctx)
    }

    fn appeared(capabilities: &str) -> Change {
        Change::Added {
            key: "container|3ab1c0f2d4e5".into(),
            after: fixture::container("/usr/local/bin/agent", capabilities, &[]),
        }
    }

    #[test]
    fn a_container_that_may_mount_filesystems_is_reported_the_first_time_it_is_seen() {
        let finding = apply(&appeared(EVERYTHING)).expect("fires");

        assert_eq!(
            finding.finding_key,
            "container|privileged|/usr/local/bin/agent"
        );
        assert_eq!(finding.kind.as_str(), "container.privileged");
        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn a_container_started_the_way_a_runtime_starts_one_says_nothing() {
        assert!(
            apply(&appeared(ORDINARY)).is_none(),
            "the default set a runtime gives a container is the set most containers have, and \
             a finding on every one of them is the noise this product dies of"
        );
    }

    #[test]
    fn a_container_that_was_already_privileged_is_not_reported_again_when_anything_else_moves() {
        let change = Change::Changed {
            key: "container|3ab1c0f2d4e5".into(),
            before: fixture::container("/usr/local/bin/agent", EVERYTHING, &[]),
            after: fixture::container("/usr/local/bin/agent", EVERYTHING, &["/srv"]),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_container_that_was_given_the_capability_between_two_readings_is_reported() {
        let change = Change::Changed {
            key: "container|3ab1c0f2d4e5".into(),
            before: fixture::container("/usr/local/bin/agent", ORDINARY, &[]),
            after: fixture::container("/usr/local/bin/agent", EVERYTHING, &[]),
        };

        assert!(apply(&change).is_some());
    }

    #[test]
    fn a_container_whose_capabilities_could_not_be_read_is_never_called_privileged() {
        let mut unreadable = fixture::container("/usr/local/bin/agent", ORDINARY, &[]);
        unreadable["capabilities_effective"] = serde_json::Value::Null;

        assert!(
            apply(&Change::Added {
                key: "container|3ab1c0f2d4e5".into(),
                after: unreadable,
            })
            .is_none(),
            "what could not be read is said in the health of the collector, not as a finding \
             about a container that may be perfectly ordinary"
        );
    }
}
