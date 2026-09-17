use serde_json::Value;

use crate::types::Kind;

pub const DROP: &str = "drop";

pub const ACCEPT: &str = "accept";

const UNSAID: &str = "—";

pub const NO_ADDRESS: &str = "no address";

pub fn what(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Ruleset => match version(item) {
            Some(version) => format!("nftables {version}"),
            None => "nftables".to_string(),
        },
        Kind::Table => format!("{} {}", text(item, "family"), text(item, "name")),
        Kind::Chain => format!(
            "{} {} · {}",
            text(item, "family"),
            text(item, "table"),
            text(item, "name")
        ),
        Kind::Interface => text(item, "name").to_string(),
        Kind::Backend => match legacy_tables(item).is_empty() {
            true => "legacy iptables".to_string(),
            false => format!("legacy iptables: {}", legacy_tables(item).join(", ")),
        },
    }
}

pub fn hook(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Chain => text(item, "hook").to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn priority(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Chain => item["priority"]
            .as_i64()
            .map(|priority| priority.to_string())
            .unwrap_or_else(|| UNSAID.to_string()),
        _ => UNSAID.to_string(),
    }
}

pub fn policy(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Chain => text(item, "policy").to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn chains(item: &Value) -> String {
    count(item, "chains")
}

pub fn rules(item: &Value) -> String {
    count(item, "rules")
}

pub fn sort_key(key: &str, item: &Value) -> (Kind, String, String) {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Chain => (
            Kind::Table,
            format!("{} {}", text(item, "family"), text(item, "table")),
            format!("1{}", text(item, "name")),
        ),
        Kind::Table => (
            Kind::Table,
            format!("{} {}", text(item, "family"), text(item, "name")),
            "0".to_string(),
        ),
        other => (other, String::new(), String::new()),
    }
}

pub fn version(item: &Value) -> Option<&str> {
    item["version"].as_str()
}

pub fn tables(item: &Value) -> u64 {
    number(item, "tables")
}

pub fn all_chains(item: &Value) -> u64 {
    number(item, "chains")
}

pub fn base_chains(item: &Value) -> u64 {
    number(item, "base_chains")
}

pub fn all_rules(item: &Value) -> u64 {
    number(item, "rules")
}

pub fn hooked_on_input(item: &Value) -> u64 {
    number(item, "hooked_on_input")
}

pub fn legacy_backend(item: &Value) -> bool {
    item["legacy_backend"].as_bool().unwrap_or(false)
}

pub fn families(item: &Value) -> Vec<&str> {
    item["families"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn legacy_tables(item: &Value) -> Vec<&str> {
    item["tables"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn addresses(item: &Value) -> Vec<&str> {
    item["addresses"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn shown_addresses(item: &Value) -> String {
    match addresses(item).is_empty() {
        true => NO_ADDRESS.to_string(),
        false => addresses(item).join(", "),
    }
}

pub fn the_way_out(item: &Value) -> bool {
    item["the_way_out"].as_bool().unwrap_or(false)
}

pub fn counted(item: &Value) -> bool {
    item["counted"].as_bool().unwrap_or(false)
}

pub fn packets(item: &Value) -> Option<u64> {
    match counted(item) {
        false => None,
        true => Some(number(item, "packets_in").saturating_add(number(item, "packets_out"))),
    }
}

pub fn flow(item: &Value, field: &str) -> u64 {
    number(item, field)
}

pub fn kept_rules(item: &Value) -> Vec<(i64, &str, &str)> {
    item["rules_kept"]
        .as_array()
        .map(|values| {
            values
                .iter()
                .map(|rule| {
                    (
                        rule["handle"].as_i64().unwrap_or(0),
                        rule["matches"].as_str().unwrap_or(UNSAID),
                        rule["does"].as_str().unwrap_or(UNSAID),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn kind_of_chain(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Ruleset) {
        Kind::Chain => text(item, "type").to_string(),
        _ => UNSAID.to_string(),
    }
}

fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap_or("?")
}

fn number(item: &Value, field: &str) -> u64 {
    item[field].as_u64().unwrap_or(0)
}

fn count(item: &Value, field: &str) -> String {
    item[field]
        .as_u64()
        .map(|count| count.to_string())
        .unwrap_or_else(|| UNSAID.to_string())
}
