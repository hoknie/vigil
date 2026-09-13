use vigil_model::{Change, Finding, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use crate::types::SocketView;
use vigil_rules::{Rule, RuleContext};

pub struct ListeningBinaryDeleted;

impl Rule for ListeningBinaryDeleted {
    fn name(&self) -> &'static str {
        "listening_binary_deleted"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let (was, now) = (SocketView::new(before), SocketView::new(after));

        if !now.is_socket() || was.executable_deleted() || !now.executable_deleted() {
            return None;
        }
        if !was.owner_resolved() || !now.owner_resolved() {
            return None;
        }

        let severity = match now.world_reachable() {
            true => Severity::Critical,
            false => Severity::High,
        };

        Some(build(
            SocketFinding {
                kind: KnownKind::ProcessBinaryDeleted,
                severity,
                rule: self.name(),
                key,
                title: format!(
                    "The binary behind {} was deleted while still listening ({})",
                    now.endpoint(),
                    now.describe_owner()
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
        ListeningBinaryDeleted.apply(change, &mut ctx)
    }

    #[test]
    fn fires_on_the_moment_the_binary_disappears() {
        let finding = apply(&Change::Changed {
            key: "tcp|0.0.0.0:5555".into(),
            before: fixture::socket("0.0.0.0", 5555, "/tmp/.hidden-nc", "root"),
            after: fixture::socket_with_deleted_binary("0.0.0.0", 5555, "/tmp/.hidden-nc"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "process.binary_deleted");
        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn stays_silent_while_the_binary_is_still_missing() {
        let deleted = fixture::socket_with_deleted_binary("0.0.0.0", 5555, "/tmp/.hidden-nc");
        let mut later = deleted.clone();
        later["uid"] = serde_json::json!(1000);

        let change = Change::Changed {
            key: "tcp|0.0.0.0:5555".into(),
            before: deleted,
            after: later,
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_binary_that_is_still_there_is_not_its_business() {
        let change = Change::Changed {
            key: "tcp|0.0.0.0:443".into(),
            before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
            after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "www-data"),
        };

        assert!(apply(&change).is_none());
    }
}
