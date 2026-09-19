use serde_json::json;
use vigil_model::Snapshot;

use super::nft_json::NftRuleset;
use crate::types::Interface;

pub const SOURCE: &str = "firewall";

pub const SUMMARY: &str = "fw-summary|nftables";

pub const LEGACY_BACKEND: &str = "fw-backend|legacy";

const TABLE: &str = "fw-table";

const CHAIN: &str = "fw-chain";

const INTERFACE: &str = "fw-interface";

pub struct FirewallReading<'a> {
    pub ruleset: &'a NftRuleset,
    pub legacy_tables: &'a [String],
    pub interfaces: &'a [Interface],
    pub counting: bool,
}

impl FirewallReading<'_> {
    pub fn held_by_the_legacy_backend(&self) -> bool {
        self.ruleset.tables.is_empty() && !self.legacy_tables.is_empty()
    }
}

pub fn firewall_snapshot(taken_at: &str, reading: &FirewallReading<'_>) -> Snapshot {
    let mut snapshot = Snapshot::new(SOURCE, taken_at.to_string());

    add_summary(&mut snapshot, reading);
    add_tables(&mut snapshot, reading);
    add_chains(&mut snapshot, reading);
    add_backend(&mut snapshot, reading);
    add_interfaces(&mut snapshot, reading.interfaces, reading.counting);

    snapshot
}

fn add_summary(snapshot: &mut Snapshot, reading: &FirewallReading<'_>) {
    let ruleset = reading.ruleset;

    snapshot.items.insert(
        SUMMARY.to_string(),
        json!({
            "version": ruleset.version,
            "families": ruleset.families(),
            "tables": ruleset.tables.len(),
            "chains": ruleset.chains(),
            "base_chains": ruleset.base_chains.len(),
            "rules": ruleset.rules,
            "hooked_on_input": ruleset.hooked_on_input(),
            "legacy_backend": reading.held_by_the_legacy_backend(),
        }),
    );
}

fn add_tables(snapshot: &mut Snapshot, reading: &FirewallReading<'_>) {
    for table in &reading.ruleset.tables {
        snapshot.items.insert(
            format!("{TABLE}|{}", table.key()),
            json!({
                "family": table.family,
                "name": table.name,
                "chains": table.chains,
                "rules": table.rules,
            }),
        );
    }
}

fn add_chains(snapshot: &mut Snapshot, reading: &FirewallReading<'_>) {
    for chain in &reading.ruleset.base_chains {
        snapshot.items.insert(
            format!("{CHAIN}|{}|{}", chain.table_key(), chain.name),
            json!({
                "family": chain.family,
                "table": chain.table,
                "name": chain.name,
                "type": chain.kind,
                "hook": chain.hook,
                "priority": chain.priority,
                "policy": chain.policy,
                "rules": chain.rules,
                "rules_kept": chain
                    .kept
                    .iter()
                    .map(|rule| json!({
                        "handle": rule.handle,
                        "matches": rule.matches,
                        "does": rule.does,
                    }))
                    .collect::<Vec<serde_json::Value>>(),
            }),
        );
    }
}

pub(super) fn add_interfaces(snapshot: &mut Snapshot, interfaces: &[Interface], counting: bool) {
    for interface in interfaces {
        let mut said = json!({
            "name": interface.name,
            "addresses": interface.addresses,
            "the_way_out": interface.the_way_out,
            "counted": counting,
        });

        if let (true, Some(traffic)) = (counting, interface.traffic) {
            said["packets_in"] = json!(traffic.packets_in);
            said["packets_out"] = json!(traffic.packets_out);
            said["bytes_in"] = json!(traffic.bytes_in);
            said["bytes_out"] = json!(traffic.bytes_out);
            said["dropped_in"] = json!(traffic.dropped_in);
            said["dropped_out"] = json!(traffic.dropped_out);
        }

        snapshot
            .items
            .insert(format!("{INTERFACE}|{}", interface.name), said);
    }
}

fn add_backend(snapshot: &mut Snapshot, reading: &FirewallReading<'_>) {
    if !reading.held_by_the_legacy_backend() {
        return;
    }

    snapshot.items.insert(
        LEGACY_BACKEND.to_string(),
        json!({
            "tables": reading.legacy_tables,
            "readable": false,
        }),
    );
}

#[cfg(test)]
mod tests {
    use vigil_model::class_of;

    use super::super::nft_json::parse_nft_ruleset;
    use super::*;

    const AT: &str = "2026-09-11T12:00:00.000Z";

    const HOST_WITH_A_FILTER: &str = r#"{"nftables": [
        {"metainfo": {"version": "1.0.6", "release_name": "Lester Gooseberry #3", "json_schema_version": 1}},
        {"table": {"family": "inet", "name": "filter", "handle": 1}},
        {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
        {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 4, "expr": [{"accept": null}]}}
    ]}"#;

    const NOTHING_AT_ALL: &str =
        r#"{"nftables": [{"metainfo": {"version": "1.0.6", "json_schema_version": 1}}]}"#;

    fn with_an_interface() -> Snapshot {
        let ruleset = parse_nft_ruleset(HOST_WITH_A_FILTER.as_bytes()).expect("reads");
        let interfaces = vec![Interface {
            name: "eth0".to_string(),
            ..Interface::default()
        }];

        firewall_snapshot(
            AT,
            &FirewallReading {
                ruleset: &ruleset,
                legacy_tables: &[],
                interfaces: &interfaces,
                counting: true,
            },
        )
    }

    fn snapshot(document: &str, legacy: &[&str]) -> Snapshot {
        let ruleset = parse_nft_ruleset(document.as_bytes()).expect("reads");
        let legacy: Vec<String> = legacy.iter().map(|name| (*name).to_string()).collect();

        firewall_snapshot(
            AT,
            &FirewallReading {
                ruleset: &ruleset,
                legacy_tables: &legacy,
                interfaces: &[],
                counting: false,
            },
        )
    }

    #[test]
    fn a_table_a_base_chain_and_the_summary_each_get_a_row_of_their_own() {
        let reading = snapshot(HOST_WITH_A_FILTER, &[]);

        assert_eq!(reading.source, SOURCE);
        assert_eq!(reading.items[SUMMARY]["tables"], 1);
        assert_eq!(reading.items[SUMMARY]["hooked_on_input"], 1);
        assert_eq!(reading.items["fw-table|inet filter"]["rules"], 1);
        assert_eq!(
            reading.items["fw-chain|inet filter|input"]["policy"],
            "drop"
        );
        assert_eq!(reading.items["fw-chain|inet filter|input"]["priority"], 0);
    }

    #[test]
    fn each_row_of_the_firewall_reading_is_a_class_of_its_own() {
        let filtering = snapshot(HOST_WITH_A_FILTER, &[]);
        let old_backend = snapshot(NOTHING_AT_ALL, &["filter"]);

        let watched = with_an_interface();
        let mut classes: Vec<&str> = filtering
            .items
            .keys()
            .chain(old_backend.items.keys())
            .chain(watched.items.keys())
            .map(|key| class_of(key))
            .collect();
        classes.sort_unstable();
        classes.dedup();

        assert_eq!(
            classes,
            vec![
                "fw-backend",
                "fw-chain",
                "fw-interface",
                "fw-summary",
                "fw-table"
            ],
            "class_of reads the first segment of the key, and the shape published beside the \
             reading has one entry per class. Four kinds of row under one class is a shape \
             where every field is optional, which asserts nothing about any of them"
        );
    }

    #[test]
    fn no_row_is_written_for_a_single_rule_however_many_there_are() {
        let reading = snapshot(HOST_WITH_A_FILTER, &[]);

        assert!(
            reading.items.keys().all(|key| !key.starts_with("fw-rule")),
            "a row per rule is a finding every time fail2ban blocks an address"
        );
        assert_eq!(reading.items.len(), 3);
    }

    #[test]
    fn a_host_whose_rules_the_legacy_backend_holds_says_so_in_a_row_of_its_own() {
        let reading = snapshot(NOTHING_AT_ALL, &["filter", "nat"]);

        assert_eq!(reading.items[SUMMARY]["legacy_backend"], true);
        assert_eq!(reading.items[LEGACY_BACKEND]["tables"][0], "filter");
        assert_eq!(reading.items[SUMMARY]["tables"], 0);
    }

    #[test]
    fn a_host_with_nftables_tables_carries_no_legacy_marker_even_when_the_old_modules_are_loaded() {
        let reading = snapshot(HOST_WITH_A_FILTER, &["filter"]);

        assert_eq!(reading.items[SUMMARY]["legacy_backend"], false);
        assert!(!reading.items.contains_key(LEGACY_BACKEND));
    }

    #[test]
    fn a_host_that_really_filters_nothing_is_a_reading_and_not_a_refusal() {
        let reading = snapshot(NOTHING_AT_ALL, &[]);

        assert_eq!(reading.items[SUMMARY]["tables"], 0);
        assert_eq!(reading.items[SUMMARY]["hooked_on_input"], 0);
        assert_eq!(reading.items[SUMMARY]["legacy_backend"], false);
        assert_eq!(reading.items.len(), 1);
    }

    #[test]
    fn the_number_of_packets_a_rule_counted_is_nowhere_in_the_reading() {
        let counted = r#"{"nftables": [
            {"metainfo": {"version": "1.0.6", "json_schema_version": 1}},
            {"table": {"family": "inet", "name": "filter", "handle": 1}},
            {"chain": {"family": "inet", "table": "filter", "name": "input", "handle": 1, "type": "filter", "hook": "input", "prio": 0, "policy": "drop"}},
            {"rule": {"family": "inet", "table": "filter", "chain": "input", "handle": 4, "expr": [{"counter": {"packets": 918, "bytes": 74216}}, {"accept": null}]}}
        ]}"#;
        let later = counted.replace("918", "2201").replace("74216", "180411");

        assert_eq!(
            snapshot(counted, &[]).items,
            snapshot(&later, &[]).items,
            "a counter moves between two readings, so a reading carrying one differs from the \
             one before it every time and the differ has something to say on every tick"
        );
    }
}
