use vigil_model::{Change, Finding, KnownKind, Severity};

use super::ssh_key_view::SshKeyView;
use crate::rules::accounts::account_finding::{AccountFinding, build, note};
use crate::{Rule, RuleContext};

pub struct SshKeyRemoved;

impl Rule for SshKeyRemoved {
    fn name(&self) -> &'static str {
        "ssh_key_removed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Removed { key, before } = change else {
            return None;
        };
        if !key.starts_with("sshkey|") {
            return None;
        }
        let view = SshKeyView::new(before);
        if !view.readable() {
            return None;
        }

        Some(build(
            AccountFinding {
                kind: KnownKind::UserSshkeyRemoved,
                severity: Severity::Low,
                rule: self.name(),
                key,
                object: "ssh_key",
                title: format!(
                    "An SSH key can no longer log in as {}: {}",
                    view.user(),
                    view.describe()
                ),
                before: Some(before.clone()),
                after: None,
                evidence: vec![note(format!("was in {}", view.source()))],
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
        SshKeyRemoved.apply(change, &mut ctx)
    }

    #[test]
    fn carries_the_same_key_the_arrival_carried_so_the_store_can_close_it() {
        let finding = apply(&Change::Removed {
            key: "sshkey|deploy|SHA256:abc".into(),
            before: fixture::ssh_key("deploy", 1000, "SHA256:abc", Some("alice@laptop"), None),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.sshkey.removed");
        assert_eq!(finding.severity, Severity::Low);
        assert_eq!(finding.finding_key, "user|sshkey|deploy|SHA256:abc");
        assert!(finding.before.is_some() && finding.after.is_none());
    }

    #[test]
    fn a_home_directory_that_became_unreadable_did_not_remove_anybodys_key() {
        let change = Change::Removed {
            key: "sshkey|alice|unreadable".into(),
            before: fixture::ssh_keys_unreadable("alice", 1001),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn an_addition_is_not_a_removal() {
        let change = Change::Added {
            key: "sshkey|deploy|SHA256:abc".into(),
            after: fixture::ssh_key("deploy", 1000, "SHA256:abc", None, None),
        };

        assert!(apply(&change).is_none());
    }
}
