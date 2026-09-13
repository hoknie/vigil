use serde_json::json;
use vigil_model::{Change, Evidence, Finding, Kind, KnownKind, Severity, State, Subject};

use vigil_rules::{Rule, RuleContext};

const DROPPING: &str = "launches|dropping";

pub struct LaunchSpoolDrained;

impl Rule for LaunchSpoolDrained {
    fn name(&self) -> &'static str {
        "launch_spool_drained"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Removed { key, before } = change else {
            return None;
        };
        if key != DROPPING {
            return None;
        }

        let now = ctx.now.clone();
        Some(Finding {
            event_id: (ctx.mint_event_id)(),
            finding_key: "agent.buffer|launches".to_string(),
            kind: Kind::Known(KnownKind::AgentBufferDrained),
            severity: Severity::Info,
            state: State::Open,
            observed_at: now.clone(),
            first_seen_at: now,
            occurrences: 1,
            title: "Program launches are being read again without any being dropped".to_string(),
            subject: Subject {
                object: "buffer".into(),
                key: json!({ "name": "launches" }),
            },
            before: Some(before.clone()),
            after: None,
            evidence: vec![Evidence {
                kind: "note".into(),
                value: "the spool is under its size limit and this reading dropped nothing. What was dropped while it was full was never read and is in no history: this says the gap has ended, never that it was filled".into(),
            }],
            redacted: Vec::new(),
            rule: Some(self.name().to_string()),
            labels: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;
    use crate::rules::LaunchSpoolDropping;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        LaunchSpoolDrained.apply(change, &mut ctx)
    }

    fn dropping() -> serde_json::Value {
        json!({ "named": false, "reason": "the audit plugin dropped the oldest events" })
    }

    #[test]
    fn a_spool_that_stopped_dropping_closes_the_finding_that_said_it_was() {
        let finding = apply(&Change::Removed {
            key: DROPPING.into(),
            before: dropping(),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "agent.buffer.drained");
        assert_eq!(finding.severity, Severity::Info);
        assert_eq!(finding.finding_key, "agent.buffer|launches");
    }

    #[test]
    fn the_two_halves_carry_one_key_so_the_second_closes_the_first_and_not_a_stranger() {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        let opened = LaunchSpoolDropping
            .apply(
                &Change::Added {
                    key: DROPPING.into(),
                    after: dropping(),
                },
                &mut ctx,
            )
            .expect("fires");
        let closed = apply(&Change::Removed {
            key: DROPPING.into(),
            before: dropping(),
        })
        .expect("fires");

        assert_eq!(opened.finding_key, closed.finding_key);
        assert_eq!(
            closed.kind,
            Kind::Known(KnownKind::AgentBufferDrained),
            "the store closes a finding by the pair of kinds on one key, and a key that \
             differed by a character would leave the open one standing for ever"
        );
        assert_eq!(
            KnownKind::AgentBufferDrained.resolves(),
            Some(KnownKind::AgentBufferDropping)
        );
    }

    #[test]
    fn nothing_else_in_the_family_is_this_rules_business() {
        assert!(
            apply(&Change::Removed {
                key: "run|alice|/usr/bin/nc".into(),
                before: fixture::launch("alice", 1000, "/usr/bin/nc"),
            })
            .is_none()
        );
        assert!(
            apply(&Change::Added {
                key: DROPPING.into(),
                after: dropping(),
            })
            .is_none(),
            "the half that opens the finding is the other rule's, and firing both on one \
             change would open and close it in the same tick"
        );
    }
}
