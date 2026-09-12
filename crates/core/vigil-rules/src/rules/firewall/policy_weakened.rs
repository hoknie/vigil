use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::firewall_finding::{FirewallFinding, build};
use super::firewall_view::{Family, FirewallView};
use crate::{Rule, RuleContext};

pub struct FirewallPolicyWeakened;

impl Rule for FirewallPolicyWeakened {
    fn name(&self) -> &'static str {
        "firewall_policy_weakened"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = FirewallView::new(key, before);
        let now = FirewallView::new(key, after);
        if !now.is(Family::Chain) {
            return None;
        }
        if !was.drops_what_no_rule_allowed() || !now.accepts_what_no_rule_allowed() {
            return None;
        }

        Some(build(
            FirewallFinding {
                kind: KnownKind::FirewallPolicyWeakened,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "firewall_chain",
                title: format!(
                    "The {} chain {} on the {} hook now accepts what no rule allowed, it dropped it before",
                    now.network_family(),
                    now.name(),
                    now.hook()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![Evidence {
                    kind: "note".into(),
                    value: format!(
                        "table {} {}, chain {}, policy {} was {}, {} rule(s) in the chain",
                        now.network_family(),
                        now.table(),
                        now.name(),
                        now.policy(),
                        was.policy(),
                        now.rules()
                    ),
                }],
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
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        FirewallPolicyWeakened.apply(change, &mut ctx)
    }

    fn policy_went(before: &str, after: &str) -> Change {
        Change::Changed {
            key: "fw-chain|inet filter|input".into(),
            before: fixture::firewall_chain("inet", "filter", "input", before),
            after: fixture::firewall_chain("inet", "filter", "input", after),
        }
    }

    #[test]
    fn a_chain_that_started_accepting_what_it_used_to_drop_is_reported_by_name() {
        let finding = apply(&policy_went("drop", "accept")).expect("fires");

        assert_eq!(finding.finding_key, "firewall|chain|inet filter|input");
        assert_eq!(finding.kind.as_str(), "firewall.policy_weakened");
        assert_eq!(finding.severity, Severity::High);
        assert!(finding.title.contains("input"), "{}", finding.title);
    }

    #[test]
    fn a_chain_that_started_dropping_what_it_used_to_accept_is_not_a_finding_of_this_rule() {
        assert!(apply(&policy_went("accept", "drop")).is_none());
    }

    #[test]
    fn a_chain_whose_rules_moved_and_whose_policy_did_not_says_nothing() {
        let mut after = fixture::firewall_chain("inet", "filter", "input", "drop");
        after["rules"] = serde_json::json!(41);

        let change = Change::Changed {
            key: "fw-chain|inet filter|input".into(),
            before: fixture::firewall_chain("inet", "filter", "input", "drop"),
            after,
        };

        assert!(apply(&change).is_none());
    }
}
