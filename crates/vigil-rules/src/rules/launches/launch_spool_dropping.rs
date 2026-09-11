use serde_json::json;
use vigil_model::{Change, Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use crate::{Rule, RuleContext};

const DROPPING: &str = "launches|dropping";

pub struct LaunchSpoolDropping;

impl Rule for LaunchSpoolDropping {
    fn name(&self) -> &'static str {
        "launch_spool_dropping"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        if key != DROPPING {
            return None;
        }

        let now = ctx.now.clone();
        Some(Finding {
            event_id: (ctx.mint_event_id)(),
            finding_key: "agent.buffer|launches".to_string(),
            kind: Kind::Known(KnownKind::AgentBufferDropping),
            severity: Severity::Medium,
            state: State::Open,
            observed_at: now.clone(),
            first_seen_at: now,
            occurrences: 1,
            title: "Program launches dropped before they were read".to_string(),
            subject: Subject {
                object: "buffer".into(),
                key: json!({ "name": "launches" }),
            },
            before: None,
            after: Some(after.clone()),
            evidence: vec![
                Evidence {
                    kind: "note".into(),
                    value: after["reason"]
                        .as_str()
                        .unwrap_or("oldest events dropped to keep the spool under its size limit")
                        .to_string(),
                },
                Evidence {
                    kind: "note".into(),
                    value: "usual cause: the daemon was not running while the plugin wrote; launches from that window are missing from this host's history, later ones are not".into(),
                },
            ],
            redacted: Vec::new(),
            rule: Some(self.name().to_string()),
            labels: Default::default(),
        })
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
        LaunchSpoolDropping.apply(change, &mut ctx)
    }

    #[test]
    fn a_dropped_window_is_reported_as_the_agent_failing_and_not_as_a_quiet_host() {
        let finding = apply(&Change::Added {
            key: DROPPING.into(),
            after: json!({ "named": false, "reason": "the audit plugin dropped the oldest events" }),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "agent.buffer.dropping");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.finding_key, "agent.buffer|launches");
    }

    #[test]
    fn nothing_else_in_the_family_is_this_rules_business() {
        assert!(
            apply(&Change::Added {
                key: "run|alice|/usr/bin/nc".into(),
                after: fixture::launch("alice", 1000, "/usr/bin/nc"),
            })
            .is_none()
        );
        assert!(
            apply(&Change::Added {
                key: "launches|capped".into(),
                after: json!({ "named": false, "reason": "…" }),
            })
            .is_none()
        );
    }
}
