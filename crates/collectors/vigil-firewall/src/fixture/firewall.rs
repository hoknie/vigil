use vigil_model::Snapshot;

use crate::parsers::{
    FirewallReading, firewall_snapshot, parse_fib_trie, parse_if_inet6, parse_ip_tables_names,
    parse_net_dev, parse_nft_ruleset, parse_route,
};
use crate::types::Interface;

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

const NET_DEV: &str = "\
Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 5140552   41220    0    0    0     0          0         0  5140552   41220    0    0    0     0       0          0
  eth0: 91200311  318841    0    7    0     0          0     51204 12084411  201773    0    2    0     0       0          0
docker0:   511204    4102    0    0    0     0          0         0  2004118    3980    0    0    0     0       0          0
";

const ROUTE: &str = "\
Iface\tDestination\tGateway \tFlags\tRefCnt\tUse\tMetric\tMask\t\tMTU\tWindow\tIRTT
eth0\t00000000\t0101A8C0\t0003\t0\t0\t100\t00000000\t0\t0\t0
eth0\t0001A8C0\t00000000\t0001\t0\t0\t100\t00FFFFFF\t0\t0\t0
docker0\t000011AC\t00000000\t0001\t0\t0\t0\t0000FFFF\t0\t0\t0
lo\t0000007F\t00000000\t0001\t0\t0\t0\t000000FF\t0\t0\t0
";

const FIB_TRIE: &str = "\
Main:
  +-- 0.0.0.0/0 3 0 5
     |-- 0.0.0.0
        /0 universe UNICAST
Local:
  +-- 0.0.0.0/1 2 0 2
     +-- 127.0.0.0/8 2 0 2
        |-- 127.0.0.1
           /32 host LOCAL
     |-- 172.17.0.1
        /32 host LOCAL
     |-- 192.168.1.23
        /32 host LOCAL
     |-- 192.168.1.255
        /32 link BROADCAST
";

const IF_INET6: &str = "\
00000000000000000000000000000001 01 80 10 80       lo
20010db8000000000000000000000042 02 40 00 80     eth0
";

pub fn firewall() -> Snapshot {
    let ruleset = parse_nft_ruleset(RULESET.as_bytes()).expect("the sample ruleset reads");
    let legacy = parse_ip_tables_names(IP_TABLES_NAMES);

    firewall_snapshot(
        "2026-09-09T09:00:00.000Z",
        &FirewallReading {
            ruleset: &ruleset,
            legacy_tables: &legacy,
            interfaces: &interfaces(),
            counting: true,
        },
    )
}

fn interfaces() -> Vec<Interface> {
    Interface::gathered(
        &parse_net_dev(NET_DEV),
        &parse_route(ROUTE),
        &parse_fib_trie(FIB_TRIE),
        &parse_if_inet6(IF_INET6),
    )
}
