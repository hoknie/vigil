use vigil_model::{Change, Finding, KnownKind, Severity};

use super::socket_finding::{SocketFinding, build, owner_evidence};
use super::socket_view::SocketView;
use crate::{Rule, RuleContext};

pub struct ListenFromWritablePath;

impl Rule for ListenFromWritablePath {
    fn name(&self) -> &'static str {
        "listen_from_writable_path"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let view = SocketView::new(after);
        if !view.is_socket() || !view.suspicious_executable() {
            return None;
        }

        let severity = match (view.executable_deleted(), view.world_reachable()) {
            (true, true) => Severity::Critical,
            _ => Severity::High,
        };

        Some(build(
            SocketFinding {
                kind: KnownKind::PortListenNew,
                severity,
                rule: self.name(),
                key,
                title: format!(
                    "New listening socket on {} held by {}",
                    view.endpoint(),
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
    use crate::RuleContext;
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-08T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ListenFromWritablePath.apply(change, &mut ctx)
    }

    #[test]
    fn a_deleted_binary_listening_to_the_world_is_the_loudest_thing_this_build_says() {
        let finding = apply(&Change::Added {
            key: "tcp|0.0.0.0:4444".into(),
            after: fixture::socket_with_deleted_binary("0.0.0.0", 4444, "/tmp/.x/nc"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert!(
            finding
                .evidence
                .iter()
                .any(|e| e.value.contains("deleted from disk")),
            "the evidence has to say what makes it critical: {:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_writable_path_on_loopback_is_high_but_not_critical() {
        let finding = apply(&Change::Added {
            key: "tcp|127.0.0.1:9001".into(),
            after: fixture::socket("127.0.0.1", 9001, "/tmp/build/server", "ci"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn an_ordinary_service_is_not_its_business() {
        let change = Change::Added {
            key: "tcp|0.0.0.0:443".into(),
            after: fixture::socket("0.0.0.0", 443, "/usr/sbin/nginx", "root"),
        };

        assert!(apply(&change).is_none());
    }
}
