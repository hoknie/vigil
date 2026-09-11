use serde_json::Value;

use super::kind::Kind;
use super::row::Row;

pub const DROP: &str = "drop";

pub const ACCEPT: &str = "accept";

const UNSAID: &str = "—";

pub fn what(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Ruleset => match version(row.item) {
            Some(version) => format!("nftables {version}"),
            None => "nftables".to_string(),
        },
        Kind::Table => format!("{} {}", text(row.item, "family"), text(row.item, "name")),
        Kind::Chain => format!(
            "{} {} · {}",
            text(row.item, "family"),
            text(row.item, "table"),
            text(row.item, "name")
        ),
        Kind::Backend => match legacy_tables(row.item).is_empty() {
            true => "legacy iptables".to_string(),
            false => format!("legacy iptables: {}", legacy_tables(row.item).join(", ")),
        },
    }
}

pub fn hook(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Chain => text(row.item, "hook").to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn priority(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Chain => row.item["priority"]
            .as_i64()
            .map(|priority| priority.to_string())
            .unwrap_or_else(|| UNSAID.to_string()),
        _ => UNSAID.to_string(),
    }
}

pub fn policy(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Chain => text(row.item, "policy").to_string(),
        _ => UNSAID.to_string(),
    }
}

pub fn chains(row: &Row<'_>) -> String {
    count(row.item, "chains")
}

pub fn rules(row: &Row<'_>) -> String {
    count(row.item, "rules")
}

pub fn sort_key(row: &Row<'_>) -> (Kind, String, String) {
    match row.kind {
        Kind::Chain => (
            Kind::Table,
            format!("{} {}", text(row.item, "family"), text(row.item, "table")),
            format!("1{}", text(row.item, "name")),
        ),
        Kind::Table => (
            Kind::Table,
            format!("{} {}", text(row.item, "family"), text(row.item, "name")),
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

pub fn kind_of_chain(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Chain => text(row.item, "type").to_string(),
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
