use serde_json::{Value, json};

pub fn firewall_ruleset(tables: u64, base_chains: u64, rules: u64) -> Value {
    json!({
        "version": "1.0.6",
        "families": match tables {
            0 => Vec::new(),
            _ => vec!["inet", "ip"],
        },
        "tables": tables,
        "chains": base_chains + tables,
        "base_chains": base_chains,
        "rules": rules,
        "hooked_on_input": match base_chains {
            0 => 0,
            _ => 1,
        },
        "legacy_backend": false,
    })
}

pub fn firewall_table(family: &str, name: &str, chains: u64, rules: u64) -> Value {
    json!({
        "family": family,
        "name": name,
        "chains": chains,
        "rules": rules,
    })
}

pub fn firewall_chain(family: &str, table: &str, name: &str, policy: &str) -> Value {
    json!({
        "family": family,
        "table": table,
        "name": name,
        "type": "filter",
        "hook": name,
        "priority": 0,
        "policy": policy,
        "rules": 3,
    })
}

pub fn firewall_legacy_backend(tables: &[&str]) -> Value {
    json!({
        "tables": tables,
        "readable": false,
    })
}
