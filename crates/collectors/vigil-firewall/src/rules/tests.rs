use serde_json::json;
use vigil_model::Change;

use super::verdict::{firewall, firewall_tick};
use crate::fixture;

#[test]
fn exactly_one_firewall_rule_fires_for_each_change_a_host_can_produce() {
    let cases: Vec<(&str, Change, &str, &str)> = vec![
        (
            "nft flush ruleset took the last chain off the input hook",
            Change::Changed {
                key: "fw-summary|nftables".into(),
                before: fixture::firewall_ruleset(2, 3, 14),
                after: fixture::firewall_ruleset(0, 0, 0),
            },
            "firewall_disabled",
            "firewall.disabled",
        ),
        (
            "the rules were put back",
            Change::Changed {
                key: "fw-summary|nftables".into(),
                before: fixture::firewall_ruleset(0, 0, 0),
                after: fixture::firewall_ruleset(2, 3, 14),
            },
            "firewall_enabled",
            "firewall.enabled",
        ),
        (
            "nft flush table emptied a table in place",
            Change::Changed {
                key: "fw-table|inet filter".into(),
                before: fixture::firewall_table("inet", "filter", 3, 9),
                after: fixture::firewall_table("inet", "filter", 3, 0),
            },
            "firewall_ruleset_flushed",
            "firewall.ruleset_flushed",
        ),
        (
            "nft delete table removed one that held rules",
            Change::Removed {
                key: "fw-table|inet filter".into(),
                before: fixture::firewall_table("inet", "filter", 3, 9),
            },
            "firewall_ruleset_flushed",
            "firewall.ruleset_flushed",
        ),
        (
            "somebody set the input policy to accept",
            Change::Changed {
                key: "fw-chain|inet filter|input".into(),
                before: fixture::firewall_chain("inet", "filter", "input", "drop"),
                after: fixture::firewall_chain("inet", "filter", "input", "accept"),
            },
            "firewall_policy_weakened",
            "firewall.policy_weakened",
        ),
    ];

    for (what, change, rule, kind) in cases {
        let fired = firewall(&change);
        assert_eq!(fired.len(), 1, "{what} fired {fired:?}");
        assert_eq!(fired[0].0, rule, "{what}");
        assert_eq!(fired[0].1, kind, "{what}");
    }
}

#[test]
fn a_flush_and_a_disabled_firewall_are_not_the_same_finding() {
    let flushed = Change::Removed {
        key: "fw-table|inet filter".into(),
        before: fixture::firewall_table("inet", "filter", 3, 9),
    };
    let no_longer_filtering = Change::Changed {
        key: "fw-summary|nftables".into(),
        before: fixture::firewall_ruleset(2, 3, 14),
        after: fixture::firewall_ruleset(0, 0, 0),
    };

    let both = firewall_tick(&[flushed, no_longer_filtering]);

    assert_eq!(
        both.len(),
        2,
        "one command empties the ruleset and takes the host off the input hook; the two facts \
         are a table that lost its rules and a host that stopped filtering, and a receiver that \
         sees only one of them cannot tell an emptied table on a still-filtering host from a \
         host with nothing left: {both:?}"
    );
    assert!(both.contains(&(
        "firewall_ruleset_flushed".to_string(),
        "firewall.ruleset_flushed".to_string()
    )));
    assert!(both.contains(&(
        "firewall_disabled".to_string(),
        "firewall.disabled".to_string()
    )));
}

#[test]
fn the_first_reading_of_this_collector_says_nothing_at_all() {
    for change in [
        Change::Added {
            key: "fw-summary|nftables".into(),
            after: fixture::firewall_ruleset(0, 0, 0),
        },
        Change::Added {
            key: "fw-table|inet filter".into(),
            after: fixture::firewall_table("inet", "filter", 3, 9),
        },
        Change::Added {
            key: "fw-chain|inet filter|input".into(),
            after: fixture::firewall_chain("inet", "filter", "input", "accept"),
        },
    ] {
        assert!(
            firewall(&change).is_empty(),
            "a host that has never been read before is not news: {change:?}"
        );
    }
}

#[test]
fn the_row_saying_the_old_backend_holds_the_rules_reaches_no_rule_at_all() {
    for change in [
        Change::Added {
            key: "fw-backend|legacy".into(),
            after: fixture::firewall_legacy_backend(&["filter", "nat"]),
        },
        Change::Removed {
            key: "fw-backend|legacy".into(),
            before: fixture::firewall_legacy_backend(&["filter"]),
        },
        Change::Changed {
            key: "fw-backend|legacy".into(),
            before: fixture::firewall_legacy_backend(&["filter"]),
            after: json!({"tables": ["filter", "nat"], "readable": false}),
        },
    ] {
        assert!(
            firewall(&change).is_empty(),
            "a marker saying this build cannot read the rules here is not a judgement about \
             them: {change:?}"
        );
    }
}

#[test]
fn a_counter_moving_under_a_rule_produces_nothing_because_the_reading_never_carried_one() {
    let change = Change::Changed {
        key: "fw-chain|inet filter|input".into(),
        before: fixture::firewall_chain("inet", "filter", "input", "drop"),
        after: fixture::firewall_chain("inet", "filter", "input", "drop"),
    };

    assert!(firewall(&change).is_empty());
}
