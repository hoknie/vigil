use vigil_model::{Change, Finding, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use crate::types::SocketView;
use vigil_rules::{Rule, RuleContext};

pub struct ListeningPortOwnerChanged;

impl Rule for ListeningPortOwnerChanged {
    fn name(&self) -> &'static str {
        "listening_port_owner_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let (was, now) = (SocketView::new(before), SocketView::new(after));

        if !now.is_socket() || !was.owner_resolved() || !now.owner_resolved() {
            return None;
        }
        if was.executable() == now.executable() && was.user() == now.user() {
            return None;
        }

        let severity = match now.suspicious_executable() {
            true => Severity::High,
            false => Severity::Medium,
        };

        Some(build(
            SocketFinding {
                kind: KnownKind::PortListenOwnerChanged,
                severity,
                rule: self.name(),
                key,
                title: format!(
                    "{} is now held by {} (was {})",
                    now.endpoint(),
                    now.describe_owner(),
                    was.describe_owner()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: owner_evidence(&now),
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use vigil_rules::RuleContext;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-08T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ListeningPortOwnerChanged.apply(change, &mut ctx)
    }

    #[test]
    fn a_different_binary_on_the_same_port_is_reported_with_both_sides() {
        let finding = apply(&Change::Changed {
            key: "tcp|0.0.0.0:443".into(),
            before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
            after: fixture::socket("0.0.0.0", 443, "/tmp/nginx", "root"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "port.listen.owner_changed");
        assert_eq!(
            finding.severity,
            Severity::High,
            "the new binary sits in /tmp"
        );
        assert!(finding.before.is_some() && finding.after.is_some());
        assert!(
            finding.title.contains("was /usr/sbin/nginx as root"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn the_same_binary_under_a_different_user_is_still_a_change_of_owner() {
        let finding = apply(&Change::Changed {
            key: "tcp|0.0.0.0:443".into(),
            before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "www-data"),
            after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Medium);
    }

    #[test]
    fn losing_the_privilege_to_look_is_not_a_change_of_owner() {
        let change = Change::Changed {
            key: "tcp|0.0.0.0:443".into(),
            before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
            after: fixture::socket_without_owner("0.0.0.0", 443),
        };

        assert!(apply(&change).is_none());
    }
}
