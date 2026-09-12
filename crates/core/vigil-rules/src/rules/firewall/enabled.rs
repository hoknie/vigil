use vigil_model::{Change, Finding, KnownKind, Severity};

use super::firewall_finding::{FirewallFinding, build, counted};
use super::firewall_view::{Family, FirewallView};
use crate::{Rule, RuleContext};

pub struct FirewallEnabled;

impl Rule for FirewallEnabled {
    fn name(&self) -> &'static str {
        "firewall_enabled"
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
        if was.hooked_on_input() > 0 || now.hooked_on_input() == 0 {
            return None;
        }

        Some(build(
            FirewallFinding {
                kind: KnownKind::FirewallEnabled,
                severity: Severity::Info,
                rule: self.name(),
                key,
                object: "firewall",
                title: format!(
                    "This host filters incoming packets again: {} chain(s) on the input hook",
                    now.hooked_on_input()
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![counted(&now)],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::firewall::FirewallDisabled;
    use crate::rules::fixture;

    fn judge(rule: &dyn Rule, change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        rule.apply(change, &mut ctx)
    }

    fn came_back() -> Change {
        Change::Changed {
            key: "fw-summary|nftables".into(),
            before: fixture::firewall_ruleset(0, 0, 0),
            after: fixture::firewall_ruleset(2, 3, 14),
        }
    }

    #[test]
    fn a_firewall_that_came_back_closes_the_finding_that_it_was_gone() {
        let gone = judge(
            &FirewallDisabled,
            &Change::Changed {
                key: "fw-summary|nftables".into(),
                before: fixture::firewall_ruleset(2, 3, 14),
                after: fixture::firewall_ruleset(0, 0, 0),
            },
        )
        .expect("the host stopped filtering");
        let back = judge(&FirewallEnabled, &came_back()).expect("and started again");

        assert_eq!(
            back.finding_key, gone.finding_key,
            "the pair closes on one key or it does not close at all"
        );
        assert_eq!(back.kind.as_str(), "firewall.enabled");
        assert_eq!(back.severity, Severity::Info);
    }

    #[test]
    fn a_host_that_was_filtering_all_along_says_nothing() {
        let change = Change::Changed {
            key: "fw-summary|nftables".into(),
            before: fixture::firewall_ruleset(2, 3, 14),
            after: fixture::firewall_ruleset(2, 3, 15),
        };

        assert!(judge(&FirewallEnabled, &change).is_none());
    }

    #[test]
    fn a_table_row_is_somebody_elses_rule() {
        let change = Change::Changed {
            key: "fw-table|inet filter".into(),
            before: fixture::firewall_table("inet", "filter", 3, 9),
            after: fixture::firewall_table("inet", "filter", 3, 0),
        };

        assert!(judge(&FirewallEnabled, &change).is_none());
    }
}
