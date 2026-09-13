use std::collections::BTreeSet;

use vigil_model::{Change, Finding};

use crate::ports::BatchRule;
use crate::ports::{Rule, RuleContext};

pub struct RuleSet {
    over_the_tick: Vec<Box<dyn BatchRule>>,
    over_one_change: Vec<Box<dyn Rule>>,
}

impl RuleSet {
    pub fn new(
        over_the_tick: Vec<Box<dyn BatchRule>>,
        over_one_change: Vec<Box<dyn Rule>>,
    ) -> Self {
        RuleSet {
            over_the_tick,
            over_one_change,
        }
    }

    pub fn of(over_one_change: Vec<Box<dyn Rule>>) -> Self {
        RuleSet::new(Vec::new(), over_one_change)
    }

    pub fn len(&self) -> usize {
        self.over_the_tick.len() + self.over_one_change.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn judge(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Vec<Finding> {
        let mut findings = Vec::new();
        let mut claimed: BTreeSet<String> = BTreeSet::new();

        for rule in &self.over_the_tick {
            let batch = rule.apply(changes, ctx);
            findings.extend(batch.findings);
            claimed.extend(batch.claimed);
        }

        for change in changes {
            if claimed.contains(change.key()) {
                continue;
            }
            for rule in &self.over_one_change {
                if let Some(finding) = rule.apply(change, ctx) {
                    findings.push(finding);
                }
            }
        }

        findings
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::{Evidence, Kind, KnownKind, Severity, State, Subject};

    use super::*;
    use crate::ports::Batch;

    struct Chatty;

    impl Rule for Chatty {
        fn name(&self) -> &'static str {
            "chatty"
        }
        fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
            Some(finding(change.key(), "chatty", ctx))
        }
    }

    struct Pairing;

    impl BatchRule for Pairing {
        fn name(&self) -> &'static str {
            "pairing"
        }
        fn apply(&self, changes: &[Change], ctx: &mut RuleContext<'_>) -> Batch {
            if changes.len() < 2 {
                return Batch::silent();
            }
            Batch {
                findings: vec![finding(changes[0].key(), "pairing", ctx)],
                claimed: changes.iter().map(|c| c.key().to_string()).collect(),
            }
        }
    }

    fn finding(key: &str, rule: &str, ctx: &mut RuleContext<'_>) -> Finding {
        Finding {
            event_id: (ctx.mint_event_id)(),
            finding_key: key.to_string(),
            kind: Kind::Known(KnownKind::PortListenNew),
            severity: Severity::Low,
            state: State::Open,
            observed_at: ctx.now.clone(),
            first_seen_at: ctx.now.clone(),
            occurrences: 1,
            title: rule.to_string(),
            subject: Subject {
                object: "socket".into(),
                key: json!({}),
            },
            before: None,
            after: None,
            evidence: Vec::<Evidence>::new(),
            redacted: Vec::new(),
            rule: Some(rule.to_string()),
            labels: Default::default(),
        }
    }

    fn judged(set: &RuleSet, changes: &[Change]) -> Vec<String> {
        let mut minted = 0;
        let mut mint = || {
            minted += 1;
            format!("event-{minted}")
        };
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        set.judge(changes, &mut ctx)
            .into_iter()
            .map(|finding| {
                format!(
                    "{}:{}",
                    finding.rule.unwrap_or_default(),
                    finding.finding_key
                )
            })
            .collect()
    }

    fn added(key: &str) -> Change {
        Change::Added {
            key: key.into(),
            after: json!({}),
        }
    }

    #[test]
    fn a_change_a_batch_rule_has_spoken_for_is_not_reported_a_second_time() {
        let set = RuleSet::new(vec![Box::new(Pairing)], vec![Box::new(Chatty)]);

        let fired = judged(&set, &[added("a"), added("b")]);

        assert_eq!(
            fired,
            vec!["pairing:a"],
            "the per-change rule must not repeat what the batch rule already said"
        );
    }

    #[test]
    fn a_change_nobody_claimed_still_reaches_every_per_change_rule() {
        let set = RuleSet::new(vec![Box::new(Pairing)], vec![Box::new(Chatty)]);

        let fired = judged(&set, &[added("a")]);

        assert_eq!(fired, vec!["chatty:a"]);
    }

    #[test]
    fn a_set_with_no_batch_rule_behaves_exactly_as_it_did_before_there_were_any() {
        let set = RuleSet::of(vec![Box::new(Chatty)]);

        assert_eq!(judged(&set, &[added("a"), added("b")]).len(), 2);
        assert_eq!(set.len(), 1);
    }
}
