use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::firewall_finding::{FirewallFinding, build};
use crate::types::{Family, FirewallView};
use vigil_rules::{Rule, RuleContext};

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
        if now.is(Family::Application) {
            if !was.blocks_all() || now.blocks_all() || !now.enabled() {
                return None;
            }
            return Some(build(
                FirewallFinding {
                    kind: KnownKind::FirewallPolicyWeakened,
                    severity: Severity::High,
                    rule: self.name(),
                    key,
                    object: "application_firewall",
                    title: "The Application Firewall of this Mac no longer blocks every incoming connection: the programs it allows accept them again".to_string(),
                    before: Some(before.clone()),
                    after: Some(after.clone()),
                    evidence: vec![Evidence {
                        kind: "note".into(),
                        value: "block all was on, and is off; which programs are allowed is on the row of the Application Firewall".into(),
                    }],
                },
                ctx,
            ));
        }
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
    use crate::fixture;

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
    fn block_all_switched_off_on_a_mac_is_a_policy_that_lets_more_in() {
        let finding = apply(&Change::Changed {
            key: "fw-application|socketfilterfw".into(),
            before: fixture::application_firewall(true, true),
            after: fixture::application_firewall(true, false),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "firewall.policy_weakened");
        assert_eq!(finding.finding_key, "firewall|application|socketfilterfw");
    }

    #[test]
    fn block_all_that_went_with_the_whole_application_firewall_is_the_other_rule() {
        assert!(
            apply(&Change::Changed {
                key: "fw-application|socketfilterfw".into(),
                before: fixture::application_firewall(true, true),
                after: fixture::application_firewall(false, false),
            })
            .is_none(),
            "one switch, one finding: firewall.disabled already says it"
        );
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
