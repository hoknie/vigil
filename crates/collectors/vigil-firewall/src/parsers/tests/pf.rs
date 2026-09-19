use super::super::pf::{
    Direction, MAIN, PfRuleset, parse_pf_anchors, parse_pf_info, parse_pf_rules,
};
use crate::fixture::{PF_ANCHORS_PRINTED, PF_INFO_DISABLED, PF_INFO_ENABLED, PF_RULES_PRINTED};

fn main(filter: &str) -> PfRuleset {
    PfRuleset {
        name: MAIN.to_string(),
        filter: parse_pf_rules(filter),
        translation: Vec::new(),
    }
}

#[test]
fn whether_pf_is_on_is_the_status_pfctl_prints_first() {
    assert_eq!(parse_pf_info(PF_INFO_ENABLED), Some(true));
    assert_eq!(parse_pf_info(PF_INFO_DISABLED), Some(false));
    assert_eq!(
        parse_pf_info("pfctl: /dev/pf: Permission denied"),
        None,
        "a refusal is not a pf that is off"
    );
}

#[test]
fn a_rule_is_read_as_what_it_does_the_way_packets_go_and_whether_it_ends_the_search() {
    let rules = parse_pf_rules(PF_RULES_PRINTED);

    assert_eq!(rules.len(), 7);
    let blocking = &rules[2];
    assert_eq!(blocking.action, "block");
    assert_eq!(blocking.direction, Direction::In);
    assert!(blocking.unconditional);
    assert!(!blocking.quick);
    assert_eq!(blocking.matches, "all");

    let loopback = &rules[4];
    assert!(loopback.quick);
    assert!(
        !loopback.unconditional,
        "a rule for one interface is not a rule for every packet"
    );
    assert_eq!(loopback.does(), "pass quick");
    assert_eq!(
        rules[1].direction,
        Direction::Both,
        "an anchor sees both ways"
    );
}

#[test]
fn what_a_ruleset_does_with_a_packet_no_rule_named_is_what_its_last_rule_for_every_packet_says() {
    let ruleset = main(PF_RULES_PRINTED);

    assert_eq!(ruleset.policy(Direction::In), "drop");
    assert_eq!(ruleset.policy(Direction::Out), "accept");
}

#[test]
fn a_later_rule_for_every_packet_overrides_an_earlier_one_unless_the_earlier_was_quick() {
    assert_eq!(
        main("block drop in all\npass in all flags S/SA keep state\n").policy(Direction::In),
        "accept",
        "pf takes the last rule that matched"
    );
    assert_eq!(
        main("block drop in quick all\npass in all\n").policy(Direction::In),
        "drop",
        "unless a quick rule stopped the search first"
    );
}

#[test]
fn a_ruleset_with_no_rule_for_every_packet_lets_in_what_nothing_named_because_pf_does() {
    assert_eq!(main("").policy(Direction::In), "accept");
    assert_eq!(
        main("block drop in proto tcp from any to any port = 23\n").policy(Direction::In),
        "accept"
    );
}

#[test]
fn the_anchors_are_every_one_pfctl_listed_under_the_main_ruleset() {
    assert_eq!(
        parse_pf_anchors(PF_ANCHORS_PRINTED),
        vec![
            "com.apple",
            "com.apple/200.AirDrop",
            "com.apple/200.AirDrop/Bonjour",
            "com.apple/250.ApplicationFirewall",
        ]
    );
    assert!(
        parse_pf_anchors("pfctl: /dev/pf: Permission denied\n-F all\n").is_empty(),
        "what pfctl said about itself is not the name of an anchor"
    );
}
