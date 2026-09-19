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

pub fn pf_ruleset(enabled: bool, hooked_on_input: u64) -> Value {
    json!({
        "enabled": enabled,
        "families": ["pf"],
        "tables": 2,
        "chains": 3,
        "base_chains": 2,
        "anchors": 1,
        "rules": 4,
        "hooked_on_input": match enabled {
            true => hooked_on_input,
            false => 0,
        },
        "legacy_backend": false,
    })
}

pub fn pf_anchor(name: &str, rules: u64) -> Value {
    json!({
        "family": "pf",
        "name": name,
        "chains": 1,
        "rules": rules,
        "anchor": true,
    })
}

pub fn application_firewall(enabled: bool, blocks_all: bool) -> Value {
    json!({
        "enabled": enabled,
        "state": match (enabled, blocks_all) {
            (false, _) => 0,
            (true, false) => 1,
            (true, true) => 2,
        },
        "blocks_all": blocks_all,
        "stealth": false,
        "allows_signed": true,
        "allows_downloaded_signed": true,
        "applications_allowed": 1,
        "applications_blocked": 0,
        "applications": [{"path": "/usr/sbin/cupsd", "allowed": true}],
    })
}
