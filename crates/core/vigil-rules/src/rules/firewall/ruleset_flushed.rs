use serde_json::Value;
use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::firewall_finding::{FirewallFinding, build};
use super::firewall_view::{Family, FirewallView};
use crate::{Rule, RuleContext};

pub struct FirewallRulesetFlushed;

impl Rule for FirewallRulesetFlushed {
    fn name(&self) -> &'static str {
        "firewall_ruleset_flushed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = emptied(change)?;
        let was = FirewallView::new(key, before);
        if !was.is(Family::Table) || was.rules() == 0 {
            return None;
        }

        let gone = after.is_none();
        Some(build(
            FirewallFinding {
                kind: KnownKind::FirewallRulesetFlushed,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "firewall_table",
                title: match gone {
                    true => format!(
                        "The {} table {} was deleted with {} rule(s) in it",
                        was.network_family(),
                        was.name(),
                        was.rules()
                    ),
                    false => format!(
                        "The {} table {} holds no rules at all: it held {}",
                        was.network_family(),
                        was.name(),
                        was.rules()
                    ),
                },
                before: Some(before.clone()),
                after: after.cloned(),
                evidence: vec![Evidence {
                    kind: "note".into(),
                    value: match gone {
                        true => "nft flush ruleset and nft delete table both leave the table absent; a reload by firewalld or a container runtime looks the same from here".into(),
                        false => "nft flush table empties the chains and leaves the table behind".into(),
                    },
                }],
            },
            ctx,
        ))
    }
}

fn emptied(change: &Change) -> Option<(&str, &Value, Option<&Value>)> {
    match change {
        Change::Removed { key, before } => Some((key, before, None)),
        Change::Changed { key, before, after } => {
            match FirewallView::new(key, after).rules() == 0 {
                true => Some((key, before, Some(after))),
                false => None,
            }
        }
        Change::Added { .. } => None,
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
        FirewallRulesetFlushed.apply(change, &mut ctx)
    }

    #[test]
    fn a_table_emptied_in_place_names_how_many_rules_it_held() {
        let finding = apply(&Change::Changed {
            key: "fw-table|inet filter".into(),
            before: fixture::firewall_table("inet", "filter", 3, 9),
            after: fixture::firewall_table("inet", "filter", 3, 0),
        })
        .expect("fires");

        assert_eq!(finding.finding_key, "firewall|table|inet filter");
        assert_eq!(finding.kind.as_str(), "firewall.ruleset_flushed");
        assert!(finding.title.contains('9'), "{}", finding.title);
    }

    #[test]
    fn a_table_deleted_outright_is_the_same_flush_as_one_emptied_in_place() {
        let finding = apply(&Change::Removed {
            key: "fw-table|inet filter".into(),
            before: fixture::firewall_table("inet", "filter", 3, 9),
        })
        .expect("nft flush ruleset removes the table, it does not empty it");

        assert_eq!(finding.kind.as_str(), "firewall.ruleset_flushed");
        assert!(finding.after.is_none());
        assert!(finding.title.contains("deleted"), "{}", finding.title);
    }

    #[test]
    fn a_table_that_never_held_a_rule_going_away_is_not_a_flush() {
        let change = Change::Removed {
            key: "fw-table|inet empty".into(),
            before: fixture::firewall_table("inet", "empty", 0, 0),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_table_that_merely_lost_some_rules_is_not_a_flush() {
        let change = Change::Changed {
            key: "fw-table|inet filter".into(),
            before: fixture::firewall_table("inet", "filter", 3, 9),
            after: fixture::firewall_table("inet", "filter", 3, 4),
        };

        assert!(
            apply(&change).is_none(),
            "fail2ban drops a rule every time a ban expires; a finding on that is noise"
        );
    }

    #[test]
    fn the_whole_ruleset_summary_is_somebody_elses_rule() {
        let change = Change::Changed {
            key: "fw-summary|nftables".into(),
            before: fixture::firewall_ruleset(2, 3, 14),
            after: fixture::firewall_ruleset(0, 0, 0),
        };

        assert!(apply(&change).is_none());
    }
}
