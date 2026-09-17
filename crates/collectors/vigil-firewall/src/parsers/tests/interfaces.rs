use vigil_model::Snapshot;

use super::super::nft_json::{RULES_KEPT_PER_CHAIN, parse_nft_ruleset};
use super::super::reading::{FirewallReading, firewall_snapshot};
use super::super::{Traffic, written_out};
use crate::types::Interface;

const AT: &str = "2026-09-16T12:00:00.000Z";

const A_FILTERING_HOST: &str = r#"{"nftables": [
    {"metainfo": {"version": "1.0.6", "json_schema_version": 1}},
    {"table": {"family": "inet", "name": "filter", "handle": 1}},
    {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
    {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 4, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 22}}, {"accept": null}]}}
]}"#;

fn eth0() -> Interface {
    Interface {
        name: "eth0".to_string(),
        addresses: vec![written_out(u32::from_be_bytes([192, 168, 1, 23]))],
        the_way_out: true,
        traffic: Some(Traffic {
            packets_in: 9_112,
            bytes_in: 1_204_881,
            dropped_in: 3,
            packets_out: 4_004,
            bytes_out: 331_207,
            dropped_out: 0,
        }),
    }
}

fn reading(document: &str, interfaces: &[Interface], counting: bool) -> Snapshot {
    let ruleset = parse_nft_ruleset(document.as_bytes()).expect("the sample ruleset reads");

    firewall_snapshot(
        AT,
        &FirewallReading {
            ruleset: &ruleset,
            legacy_tables: &[],
            interfaces,
            counting,
        },
    )
}

#[test]
fn an_interface_of_this_host_is_a_row_of_its_own_with_the_address_it_answers_on() {
    let read = reading(A_FILTERING_HOST, &[eth0()], false);
    let interface = &read.items["fw-interface|eth0"];

    assert_eq!(interface["name"], "eth0");
    assert_eq!(interface["addresses"][0], "192.168.1.23");
    assert_eq!(interface["the_way_out"], true);
}

#[test]
fn what_went_through_an_interface_is_left_out_until_this_host_is_told_to_count_it() {
    let quiet = reading(A_FILTERING_HOST, &[eth0()], false);
    let watched = reading(A_FILTERING_HOST, &[eth0()], true);

    assert_eq!(quiet.items["fw-interface|eth0"]["counted"], false);
    assert!(
        quiet.items["fw-interface|eth0"]["packets_in"].is_null(),
        "an interface counter moves every time a packet arrives, so a reading that always \
         carried one would differ from the one before it on every tick for the life of the \
         host"
    );
    assert_eq!(watched.items["fw-interface|eth0"]["packets_in"], 9_112);
    assert_eq!(watched.items["fw-interface|eth0"]["packets_out"], 4_004);
    assert_eq!(watched.items["fw-interface|eth0"]["dropped_in"], 3);
}

#[test]
fn a_host_not_told_to_count_reads_the_same_twice_however_much_traffic_passed() {
    let first = reading(A_FILTERING_HOST, &[eth0()], false);
    let mut busier = eth0();
    busier.traffic = Some(Traffic {
        packets_in: 5_000_000,
        ..busier.traffic.expect("the sample counts")
    });

    assert_eq!(
        first.items,
        reading(A_FILTERING_HOST, &[busier], false).items,
        "the switch is the whole of what keeps this reading still, and a reading that moves \
         anyway puts a change line in the journal once a minute on every host the agent is \
         installed on"
    );
}

#[test]
fn a_chain_carries_what_its_rules_match_and_what_they_do_without_a_row_for_any_of_them() {
    let read = reading(A_FILTERING_HOST, &[], false);
    let kept = &read.items["fw-chain|inet filter|input"]["rules_kept"];

    assert_eq!(kept[0]["handle"], 4);
    assert_eq!(kept[0]["matches"], "tcp dport 22");
    assert_eq!(kept[0]["does"], "accept");
    assert!(
        read.items.keys().all(|key| !key.starts_with("fw-rule")),
        "the detail of a rule hangs off the chain that holds it; a row per rule would be a \
         finding every time fail2ban blocks an address"
    );
}

#[test]
fn a_chain_of_a_thousand_rules_carries_the_first_of_them_and_not_the_thousand() {
    let read = reading(&generated(1_000), &[], false);
    let chain = &read.items["fw-chain|inet filter|input"];

    assert_eq!(chain["rules"], 1_001);
    assert_eq!(
        chain["rules_kept"].as_array().map(Vec::len),
        Some(RULES_KEPT_PER_CHAIN),
        "fail2ban and docker write chains of any length, and a reading that keeps every rule \
         of one is a reading nobody can afford to send to the console twice a second"
    );
}

fn generated(rules: usize) -> String {
    let mut document = String::from(A_FILTERING_HOST.trim_end().trim_end_matches("]}"));
    for handle in 0..rules {
        document.push_str(&format!(
            ",\n{{\"rule\": {{\"family\": \"inet\", \"table\": \"filter\", \"chain\": \"input\", \"handle\": {}, \"expr\": [{{\"drop\": null}}]}}}}",
            handle + 100
        ));
    }
    document.push_str("]}");
    document
}
