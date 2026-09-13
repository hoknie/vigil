use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use crate::types::GroupView;
use vigil_rules::{Rule, RuleContext};

pub struct PrivilegedGroupMemberAdded;

impl Rule for PrivilegedGroupMemberAdded {
    fn name(&self) -> &'static str {
        "privileged_group_member_added"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        if !key.starts_with("group|") {
            return None;
        }

        let now = GroupView::new(after);
        if !now.privileged() {
            return None;
        }

        let empty = serde_json::json!({ "members": [] });
        let was = GroupView::new(before.unwrap_or(&empty));
        let gained = now.members_gained_since(&was);
        if gained.is_empty() {
            return None;
        }

        let reason = now.privilege().unwrap_or("administrative privileges");
        Some(build(
            AccountFinding {
                kind: KnownKind::UserGroupPrivilegedMemberAdded,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "group",
                title: format!(
                    "{} joined the group {} ({reason})",
                    gained.join(", "),
                    now.name()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    note(format!(
                        "group {} now holds {}",
                        now.name(),
                        now.members().join(", ")
                    )),
                    note(reason),
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
        PrivilegedGroupMemberAdded.apply(change, &mut ctx)
    }

    #[test]
    fn a_new_member_of_sudo_is_reported_with_the_group_it_joined() {
        let finding = apply(&Change::Changed {
            key: "group|sudo".into(),
            before: fixture::group("sudo", &["alice"], Some("may run commands as any user")),
            after: fixture::group(
                "sudo",
                &["alice", "deploy"],
                Some("may run commands as any user"),
            ),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.group.privileged_member_added");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.finding_key, "user|group|sudo");
        assert!(
            finding.title.starts_with("deploy joined"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn the_title_says_why_docker_belongs_in_this_list() {
        let finding = apply(&Change::Changed {
            key: "group|docker".into(),
            before: fixture::group(
                "docker",
                &[],
                Some("mounts the host filesystem — root by another route"),
            ),
            after: fixture::group(
                "docker",
                &["deploy"],
                Some("mounts the host filesystem — root by another route"),
            ),
        })
        .expect("fires");

        assert!(
            finding.title.contains("root by another route"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn an_ordinary_group_is_not_its_business() {
        let change = Change::Changed {
            key: "group|developers".into(),
            before: fixture::group("developers", &["alice"], None),
            after: fixture::group("developers", &["alice", "deploy"], None),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_privileged_group_appearing_with_nobody_in_it_is_a_package_being_installed() {
        let change = Change::Added {
            key: "group|docker".into(),
            after: fixture::group("docker", &[], Some("root by another route")),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn somebody_leaving_a_privileged_group_is_not_somebody_joining_one() {
        let change = Change::Changed {
            key: "group|sudo".into(),
            before: fixture::group(
                "sudo",
                &["alice", "deploy"],
                Some("may run commands as any user"),
            ),
            after: fixture::group("sudo", &["alice"], Some("may run commands as any user")),
        };

        assert!(apply(&change).is_none());
    }
}
