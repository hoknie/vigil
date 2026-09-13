use vigil_model::{Change, Finding, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use crate::types::SocketView;
use vigil_rules::{Rule, RuleContext};

pub struct ClosedListeningPort;

impl Rule for ClosedListeningPort {
    fn name(&self) -> &'static str {
        "closed_listening_port"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Removed { key, before } = change else {
            return None;
        };
        let view = SocketView::new(before);
        if !view.is_network_socket() {
            return None;
        }

        Some(build(
            SocketFinding {
                kind: KnownKind::PortListenRemoved,
                severity: Severity::Low,
                rule: self.name(),
                key,
                title: format!(
                    "Listening socket on {}:{} is gone ({})",
                    view.address(),
                    view.port(),
                    view.describe_owner()
                ),
                before: Some(before.clone()),
                after: None,
                evidence: owner_evidence(&view),
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
        ClosedListeningPort.apply(change, &mut ctx)
    }

    #[test]
    fn reports_the_disappearance_with_what_used_to_hold_it() {
        let finding = apply(&Change::Removed {
            key: "tcp|127.0.0.1:5432".into(),
            before: fixture::socket(
                "127.0.0.1",
                5432,
                "/usr/lib/postgresql/15/bin/postgres",
                "postgres",
            ),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "port.listen.removed");
        assert_eq!(finding.severity, Severity::Low);
        assert!(finding.before.is_some());
        assert!(
            finding.after.is_none(),
            "there is no 'after' for a thing that is gone"
        );
    }

    #[test]
    fn an_addition_is_not_a_removal() {
        let change = Change::Added {
            key: "tcp|0.0.0.0:443".into(),
            after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
