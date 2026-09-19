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
        if now.is(Family::Application) {
            return switched_off(self.name(), key, before, after, ctx);
        }
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
                title: match (now.is_pf(), was.enabled() && !now.enabled()) {
                    (true, true) => "pf on this Mac was switched off: nothing it held filters incoming packets now".to_string(),
                    (true, false) => format!(
                        "pf on this Mac no longer filters incoming packets: no rule of its main ruleset applies to them, {} did",
                        was.hooked_on_input()
                    ),
                    (false, _) => format!(
                        "This host no longer filters incoming packets: {} chain(s) on the input hook, was {}",
                        now.hooked_on_input(),
                        was.hooked_on_input()
                    ),
                },
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

fn switched_off(
    rule: &'static str,
    key: &str,
    before: &serde_json::Value,
    after: &serde_json::Value,
    ctx: &mut RuleContext<'_>,
) -> Option<Finding> {
    let was = FirewallView::new(key, before);
    let now = FirewallView::new(key, after);
    if !was.enabled() || now.enabled() {
        return None;
    }

    Some(build(
        FirewallFinding {
            kind: KnownKind::FirewallDisabled,
            severity: Severity::High,
            rule,
            key,
            object: "application_firewall",
            title: "The Application Firewall of this Mac was switched off".to_string(),
            before: Some(before.clone()),
            after: Some(after.clone()),
            evidence: vec![Evidence {
                kind: "note".into(),
                value: "every program on this Mac that listens is now reachable by anything that can route to it, unless pf filters it".into(),
            }],
        },
        ctx,
    ))
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
    fn pf_switched_off_on_a_mac_is_reported_under_the_key_of_the_pf_ruleset() {
        let finding = apply(&Change::Changed {
            key: "fw-summary|pf".into(),
            before: fixture::pf_ruleset(true, 1),
            after: fixture::pf_ruleset(false, 1),
        })
        .expect("fires");

        assert_eq!(finding.finding_key, "firewall|summary|pf");
        assert_eq!(finding.kind.as_str(), "firewall.disabled");
        assert!(
            finding.title.contains("pf on this Mac was switched off"),
            "{}",
            finding.title
        );
        assert_eq!(finding.subject.key["backend"], "pf");
    }

    #[test]
    fn the_application_firewall_switched_off_is_a_host_that_stopped_filtering() {
        let finding = apply(&Change::Changed {
            key: "fw-application|socketfilterfw".into(),
            before: fixture::application_firewall(true, false),
            after: fixture::application_firewall(false, false),
        })
        .expect("fires");

        assert_eq!(finding.finding_key, "firewall|application|socketfilterfw");
        assert_eq!(finding.subject.object, "application_firewall");
        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn an_application_firewall_that_was_off_and_stays_off_says_nothing() {
        assert!(
            apply(&Change::Changed {
                key: "fw-application|socketfilterfw".into(),
                before: fixture::application_firewall(false, false),
                after: fixture::application_firewall(false, true),
            })
            .is_none()
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
