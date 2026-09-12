use vigil_model::{Change, Finding, KnownKind, Severity};

use super::ssh_key_view::SshKeyView;
use crate::rules::accounts::account_finding::{AccountFinding, build, note};
use crate::{Rule, RuleContext};

pub struct SshKeyAdded;

impl Rule for SshKeyAdded {
    fn name(&self) -> &'static str {
        "ssh_key_added"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        if !key.starts_with("sshkey|") {
            return None;
        }
        let view = SshKeyView::new(after);
        if !view.readable() {
            return None;
        }

        let severity = match view.uid() == 0 {
            true => Severity::Critical,
            false => Severity::High,
        };

        let mut evidence = vec![
            note(format!("in {}", view.source())),
            note(format!(
                "fingerprint {}; compare with `ssh-keygen -lf {}`",
                view.fingerprint(),
                view.source()
            )),
        ];
        evidence.push(match view.options() {
            Some(options) => note(format!("restricted by {options}")),
            None => note("no command, source or expiry restriction"),
        });

        Some(build(
            AccountFinding {
                kind: KnownKind::UserSshkeyAdded,
                severity,
                rule: self.name(),
                key,
                object: "ssh_key",
                title: format!(
                    "A new SSH key may log in as {}: {}",
                    view.user(),
                    view.describe()
                ),
                before: None,
                after: Some(after.clone()),
                evidence,
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
        SshKeyAdded.apply(change, &mut ctx)
    }

    #[test]
    fn reports_the_fingerprint_and_the_comment_and_never_the_key() {
        let finding = apply(&Change::Added {
            key: "sshkey|deploy|SHA256:abc".into(),
            after: fixture::ssh_key("deploy", 1000, "SHA256:abc", Some("alice@laptop"), None),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.sshkey.added");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.finding_key, "user|sshkey|deploy|SHA256:abc");
        assert!(finding.title.contains("alice@laptop"), "{}", finding.title);

        let printed = serde_json::to_string(&finding).expect("serialises");
        assert!(
            !printed.contains("AAAAC3Nza"),
            "the key body is not carried"
        );
    }

    #[test]
    fn a_key_that_logs_in_as_root_is_the_host_rather_than_an_account_on_it() {
        let finding = apply(&Change::Added {
            key: "sshkey|root|SHA256:abc".into(),
            after: fixture::ssh_key("root", 0, "SHA256:abc", None, None),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn the_absence_of_a_restriction_is_said_out_loud() {
        let unrestricted = apply(&Change::Added {
            key: "sshkey|deploy|SHA256:abc".into(),
            after: fixture::ssh_key("deploy", 1000, "SHA256:abc", None, None),
        })
        .expect("fires");
        let pinned = apply(&Change::Added {
            key: "sshkey|deploy|SHA256:def".into(),
            after: fixture::ssh_key(
                "deploy",
                1000,
                "SHA256:def",
                None,
                Some("command=\"/usr/bin/backup\",from=\"10.0.0.0/8\""),
            ),
        })
        .expect("fires");

        assert!(
            unrestricted.evidence.iter().any(|line| line
                .value
                .contains("no command, source or expiry restriction")),
            "{:?}",
            unrestricted.evidence
        );
        assert!(
            pinned
                .evidence
                .iter()
                .any(|line| line.value.contains("restricted by")),
            "{:?}",
            pinned.evidence
        );
    }

    #[test]
    fn a_home_directory_the_agent_may_not_read_does_not_look_like_a_new_key() {
        let change = Change::Added {
            key: "sshkey|alice|unreadable".into(),
            after: fixture::ssh_keys_unreadable("alice", 1001),
        };

        assert!(apply(&change).is_none());
    }
}
