use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::firewall_finding::{FirewallFinding, build, counted};
use crate::types::{Family, FirewallView};
use vigil_rules::{Rule, RuleContext};

pub struct FirewallDisabled;

impl Rule for FirewallDisabled {
    fn name(&self) -> &'static str {
        "firewall_disabled"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = FirewallView::new(key, before);
        let now = FirewallView::new(key, after);
        if !now.is(Family::Ruleset) {
            return None;
        }
        if was.hooked_on_input() == 0 || now.hooked_on_input() > 0 {
            return None;
        }
        if now.legacy_backend() {
            return None;
        }

        Some(build(
            FirewallFinding {
                kind: KnownKind::FirewallDisabled,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "firewall",
                title: format!(
                    "This host no longer filters incoming packets: {} chain(s) on the input hook, was {}",
                    now.hooked_on_input(),
                    was.hooked_on_input()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    counted(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "every listening socket on this host is now reachable by anything that can route to it".into(),
                    },
                ],
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
        FirewallDisabled.apply(change, &mut ctx)
    }

    fn ruleset_changed(before: serde_json::Value, after: serde_json::Value) -> Change {
        Change::Changed {
            key: "fw-summary|nftables".into(),
            before,
            after,
        }
    }

    #[test]
    fn a_host_that_stopped_filtering_is_reported_under_a_key_a_person_can_suppress() {
        let finding = apply(&ruleset_changed(
            fixture::firewall_ruleset(2, 3, 14),
            fixture::firewall_ruleset(0, 0, 0),
        ))
        .expect("fires");

        assert_eq!(finding.finding_key, "firewall|summary|nftables");
        assert_eq!(finding.kind.as_str(), "firewall.disabled");
        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn a_host_whose_rules_the_old_backend_holds_is_never_called_a_host_without_a_firewall() {
        let mut after = fixture::firewall_ruleset(0, 0, 0);
        after["legacy_backend"] = serde_json::json!(true);

        assert!(
            apply(&ruleset_changed(fixture::firewall_ruleset(2, 3, 14), after)).is_none(),
            "nftables lists nothing because the rules are in the old backend; saying the \
             firewall is off on a production host is worse than saying nothing"
        );
    }

    #[test]
    fn a_first_reading_says_nothing_because_it_has_nothing_to_compare_with() {
        let change = Change::Added {
            key: "fw-summary|nftables".into(),
            after: fixture::firewall_ruleset(0, 0, 0),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_rule_count_that_moved_without_the_input_hook_going_is_not_this_finding() {
        let change = ruleset_changed(
            fixture::firewall_ruleset(2, 3, 14),
            fixture::firewall_ruleset(2, 3, 19),
        );

        assert!(apply(&change).is_none());
    }
}
