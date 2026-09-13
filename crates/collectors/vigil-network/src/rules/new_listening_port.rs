use vigil_model::{Change, Finding, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use crate::types::SocketView;
use vigil_rules::{Rule, RuleContext};

pub struct NewListeningPort;

impl Rule for NewListeningPort {
    fn name(&self) -> &'static str {
        "new_listening_port"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = SocketView::new(after);
        if !view.is_network_socket() || view.suspicious_executable() {
            return None;
        }

        let severity = match view.world_reachable() {
            true => Severity::Medium,
            false => Severity::Low,
        };

        Some(build(
            SocketFinding {
                kind: KnownKind::PortListenNew,
                severity,
                rule: self.name(),
                key,
                title: format!(
                    "New listening socket on {}:{} ({})",
                    view.address(),
                    view.port(),
                    view.describe_owner()
                ),
                before: None,
                after: Some(after.clone()),
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
        NewListeningPort.apply(change, &mut ctx)
    }

    #[test]
    fn a_port_the_network_can_reach_outranks_one_bound_to_loopback() {
        let exposed = apply(&Change::Added {
            key: "tcp|0.0.0.0:8080".into(),
            after: fixture::socket("0.0.0.0", 8080, "/usr/bin/python3", "deploy"),
        })
        .expect("fires");
        let local = apply(&Change::Added {
            key: "tcp|127.0.0.1:8080".into(),
            after: fixture::socket("127.0.0.1", 8080, "/usr/bin/python3", "deploy"),
        })
        .expect("fires");

        assert_eq!(exposed.severity, Severity::Medium);
        assert_eq!(local.severity, Severity::Low);
        assert_eq!(exposed.finding_key, "port.listen|tcp|0.0.0.0:8080");
        assert!(
            exposed.title.contains("/usr/bin/python3 as deploy"),
            "{}",
            exposed.title
        );
    }

    #[test]
    fn it_stays_quiet_about_a_binary_the_other_rule_speaks_about() {
        let change = Change::Added {
            key: "tcp|0.0.0.0:4444".into(),
            after: fixture::socket("0.0.0.0", 4444, "/dev/shm/payload", "www-data"),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_removal_is_not_a_new_port() {
        let change = Change::Removed {
            key: "tcp|0.0.0.0:443".into(),
            before: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
