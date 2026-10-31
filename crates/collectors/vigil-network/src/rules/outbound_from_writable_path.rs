use serde_json::json;
use vigil_model::{Change, Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::types::PeerView;
use vigil_rules::{Rule, RuleContext};

const FINDING_KEY_PREFIX: &str = "connection.outbound";

pub struct OutboundFromWritablePath;

impl Rule for OutboundFromWritablePath {
    fn name(&self) -> &'static str {
        "outbound_from_writable_path"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        let pair = PeerView::new(key, after);
        if !pair.is_outbound() || !pair.suspicious_program() {
            return None;
        }

        let program = pair.program().unwrap_or("a program");
        let severity = match pair.program_deleted() {
            true => Severity::Critical,
            false => Severity::High,
        };
        let path = match pair.program_deleted() {
            true => format!("{program} (deleted from disk while still running)"),
            false => program.to_string(),
        };
        let now = ctx.now.clone();

        Some(Finding {
            event_id: (ctx.mint_event_id)(),
            finding_key: format!(
                "{FINDING_KEY_PREFIX}|{}",
                key.strip_prefix("outbound|").unwrap_or(key)
            ),
            kind: Kind::Known(KnownKind::ConnectionFromWritablePath),
            severity,
            state: State::Open,
            observed_at: now.clone(),
            first_seen_at: now,
            occurrences: 1,
            title: format!(
                "{program} as {} is talking to {}:{}",
                pair.user(),
                pair.remote_address(),
                pair.remote_port()
            ),
            subject: Subject {
                object: "connection".into(),
                key: json!({
                    "protocol": pair.protocol(),
                    "remote_address": pair.remote_address(),
                    "remote_port": pair.remote_port(),
                    "executable": pair.program(),
                }),
            },
            before: None,
            after: Some(after.clone()),
            evidence: vec![Evidence {
                kind: "path".into(),
                value: path,
            }],
            redacted: Vec::new(),
            rule: Some(self.name().to_string()),
            labels: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn pair(program: Option<&str>, deleted: bool) -> (String, Value) {
        let named = program.unwrap_or("unresolved");
        (
            format!("outbound|tcp|203.0.113.7:4444|{named}"),
            json!({
                "protocol": "tcp", "remote_address": "203.0.113.7", "remote_port": 4444,
                "program": program, "program_deleted": deleted, "uid": 33, "user": "www-data",
                "owner_resolved": program.is_some(),
            }),
        )
    }

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-10-31T12:00:00.000Z".into(),
            mint_event_id: &mut mint,
        };
        OutboundFromWritablePath.apply(change, &mut ctx)
    }

    fn added(program: Option<&str>, deleted: bool) -> Change {
        let (key, after) = pair(program, deleted);
        Change::Added { key, after }
    }

    #[test]
    fn a_deleted_binary_in_tmp_talking_out_is_critical() {
        let finding = apply(&added(Some("/tmp/.x/nc"), true)).expect("fires");

        assert_eq!(finding.kind.as_str(), "connection.from_writable_path");
        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(
            finding.finding_key,
            "connection.outbound|tcp|203.0.113.7:4444|/tmp/.x/nc"
        );
        assert_eq!(
            finding.title,
            "/tmp/.x/nc as www-data is talking to 203.0.113.7:4444"
        );
        assert_eq!(
            finding.evidence[0].value,
            "/tmp/.x/nc (deleted from disk while still running)"
        );
        assert_eq!(
            finding.program(),
            Some("/tmp/.x/nc"),
            "a suppression naming the program covers this finding as it covers its ports"
        );
    }

    #[test]
    fn a_program_in_home_talking_out_is_high() {
        let finding = apply(&added(Some("/home/ci/agent"), false)).expect("fires");

        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn a_new_peer_of_an_ordinary_program_says_nothing() {
        assert!(apply(&added(Some("/usr/bin/curl"), false)).is_none());
    }

    #[test]
    fn a_pair_that_went_away_says_nothing() {
        let (key, before) = pair(Some("/tmp/.x/nc"), true);

        assert!(apply(&Change::Removed { key, before }).is_none());
    }

    #[test]
    fn a_pair_with_no_resolved_owner_says_nothing() {
        assert!(apply(&added(None, false)).is_none());
    }

    #[test]
    fn an_inbound_pair_raises_no_finding() {
        let change = Change::Added {
            key: "inbound|tcp|443|198.51.100.5|/tmp/.x/server".into(),
            after: json!({
                "protocol": "tcp", "local_port": 443, "remote_address": "198.51.100.5",
                "program": "/tmp/.x/server", "program_deleted": true, "uid": 33,
                "user": "www-data", "owner_resolved": true,
            }),
        };

        assert!(apply(&change).is_none());
    }
}
