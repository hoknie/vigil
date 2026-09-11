use serde_json::json;
use vigil_model::Snapshot;

const AT: &str = "2026-09-09T09:00:00.000Z";

pub fn firewall() -> Snapshot {
    Snapshot::new("firewall", AT)
        .with(
            "fw-summary|nftables",
            json!({
                "version": "1.0.6",
                "families": ["inet", "ip"],
                "tables": 2,
                "chains": 7,
                "base_chains": 5,
                "rules": 7,
                "hooked_on_input": 1,
                "legacy_backend": false,
            }),
        )
        .with(
            "fw-table|inet filter",
            json!({"family": "inet", "name": "filter", "chains": 4, "rules": 4}),
        )
        .with(
            "fw-table|ip nat",
            json!({"family": "ip", "name": "nat", "chains": 3, "rules": 3}),
        )
        .with(
            "fw-chain|inet filter|input",
            json!({
                "family": "inet", "table": "filter", "name": "input", "type": "filter",
                "hook": "input", "priority": 0, "policy": "drop", "rules": 3,
            }),
        )
        .with(
            "fw-chain|inet filter|forward",
            json!({
                "family": "inet", "table": "filter", "name": "forward", "type": "filter",
                "hook": "forward", "priority": 0, "policy": "drop", "rules": 0,
            }),
        )
        .with(
            "fw-chain|inet filter|output",
            json!({
                "family": "inet", "table": "filter", "name": "output", "type": "filter",
                "hook": "output", "priority": 0, "policy": "accept", "rules": 0,
            }),
        )
        .with(
            "fw-chain|ip nat|PREROUTING",
            json!({
                "family": "ip", "table": "nat", "name": "PREROUTING", "type": "nat",
                "hook": "prerouting", "priority": -100, "policy": "accept", "rules": 1,
            }),
        )
        .with(
            "fw-chain|ip nat|POSTROUTING",
            json!({
                "family": "ip", "table": "nat", "name": "POSTROUTING", "type": "nat",
                "hook": "postrouting", "priority": 100, "policy": "accept", "rules": 1,
            }),
        )
}

pub fn held_by_the_old_backend() -> Snapshot {
    Snapshot::new("firewall", AT)
        .with(
            "fw-summary|nftables",
            json!({
                "version": "1.0.6",
                "families": [],
                "tables": 0,
                "chains": 0,
                "base_chains": 0,
                "rules": 0,
                "hooked_on_input": 0,
                "legacy_backend": true,
            }),
        )
        .with(
            "fw-backend|legacy",
            json!({"tables": ["filter", "nat"], "readable": false}),
        )
}

pub fn filtering_nothing() -> Snapshot {
    Snapshot::new("firewall", AT).with(
        "fw-summary|nftables",
        json!({
            "version": "1.0.6",
            "families": [],
            "tables": 0,
            "chains": 0,
            "base_chains": 0,
            "rules": 0,
            "hooked_on_input": 0,
            "legacy_backend": false,
        }),
    )
}
