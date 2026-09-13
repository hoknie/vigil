use vigil_model::Snapshot;

use crate::parsers::{
    FirewallReading, firewall_snapshot, parse_ip_tables_names, parse_nft_ruleset,
};

const RULESET: &str = r#"{"nftables": [
    {"metainfo": {"version": "1.0.6", "release_name": "Lester Gooseberry #3", "json_schema_version": 1}},
    {"table": {"family": "inet", "name": "filter", "handle": 1}},
    {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
    {"chain": {"family": "inet", "table": "filter", "name": "forward", "handle": 2, "type": "filter", "hook": "forward", "prio": 0, "policy": "drop"}},
    {"chain": {"family": "inet", "table": "filter", "name": "output", "handle": 3, "type": "filter", "hook": "output", "prio": 0, "policy": "accept"}},
    {"chain": {"family": "inet", "table": "filter", "name": "allowed", "handle": 4}},
    {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 10, "expr": [{"match": {"op": "==", "left": {"meta": {"key": "iifname"}}, "right": "lo"}}, {"accept": null}]}},
    {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 11, "expr": [{"match": {"op": "in", "left": {"ct": {"key": "state"}}, "right": ["established", "related"]}}, {"accept": null}]}},
    {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 12, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 22}}, {"jump": {"target": "allowed"}}]}},
    {"rule": {"family": "inet", "table": "filter", "chain": "allowed", "handle": 13, "expr": [{"counter": {"packets": 4182, "bytes": 291402}}, {"accept": null}]}},
    {"table": {"family": "ip", "name": "nat", "handle": 2}},
    {"chain": {"family": "ip", "table": "nat", "name": "DOCKER", "handle": 1}},
    {"chain": {"family": "ip", "table": "nat", "name": "PREROUTING", "handle": 2, "type": "nat", "hook": "prerouting", "prio": -100, "policy": "accept"}},
    {"chain": {"family": "ip", "table": "nat", "name": "POSTROUTING", "handle": 3, "type": "nat", "hook": "postrouting", "prio": 100, "policy": "accept"}},
    {"rule": {"family": "ip", "table": "nat", "chain": "PREROUTING", "handle": 20, "expr": [{"match": {"op": "in", "left": {"meta": {"key": "nfproto"}}, "right": "ipv4"}}, {"jump": {"target": "DOCKER"}}]}},
    {"rule": {"family": "ip", "table": "nat", "chain": "POSTROUTING", "handle": 21, "expr": [{"match": {"op": "!=", "left": {"meta": {"key": "oifname"}}, "right": "docker0"}}, {"masquerade": null}]}},
    {"rule": {"family": "ip", "table": "nat", "chain": "DOCKER", "handle": 22, "expr": [{"match": {"op": "==", "left": {"payload": {"protocol": "tcp", "field": "dport"}}, "right": 8080}}, {"dnat": {"addr": "172.17.0.2", "port": 80}}]}}
]}"#;

const IP_TABLES_NAMES: &str = "";

pub fn firewall() -> Snapshot {
    let ruleset = parse_nft_ruleset(RULESET.as_bytes()).expect("the sample ruleset reads");
    let legacy = parse_ip_tables_names(IP_TABLES_NAMES);

    firewall_snapshot(
        "2026-09-09T09:00:00.000Z",
        &FirewallReading {
            ruleset: &ruleset,
            legacy_tables: &legacy,
        },
    )
}
