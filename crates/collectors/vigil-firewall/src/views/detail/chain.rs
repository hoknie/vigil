use serde_json::Value;
use vigil_view::Piece;

use super::super::fields::{ACCEPT, DROP};

pub(super) fn chain(key: &str, item: &Value) -> Vec<Piece> {
    let family = item["family"].as_str().unwrap_or("?");
    let table = item["table"].as_str().unwrap_or("?");
    let name = item["name"].as_str().unwrap_or("?");
    let policy = item["policy"].as_str().unwrap_or("?");

    let mut said = vec![
        Piece::title("CHAIN", format!("{family} {table} · {name}")),
        Piece::Blank,
    ];

    for (label, value) in [
        ("family", family.to_string()),
        ("table", table.to_string()),
        ("name", name.to_string()),
        ("type", item["type"].as_str().unwrap_or("?").to_string()),
        ("hook", item["hook"].as_str().unwrap_or("?").to_string()),
        ("priority", number(item, "priority")),
        ("policy", policy.to_string()),
        ("rules", number(item, "rules")),
        ("object", key.to_string()),
    ] {
        said.push(Piece::field(label, value));
    }
    said.push(Piece::Blank);

    said.push(Piece::heading("WHAT THE POLICY MEANS"));
    let means = match policy {
        DROP => {
            "drop: a packet this chain saw and no rule in it allowed is thrown away. The \
             chain is closed by default and the rules open it."
        }
        ACCEPT => {
            "accept: a packet this chain saw and no rule in it dropped goes through. The \
             chain is open by default and the rules close it."
        }
        _ => {
            "This chain is not a base chain, or the reading carries no policy for it: a \
             regular chain is only reached by a jump from one that is."
        }
    };
    said.push(
        match policy == ACCEPT && item["hook"].as_str() == Some("input") {
            true => Piece::warning(means),
            false => Piece::text(means),
        },
    );
    said.push(Piece::Blank);

    said
}

pub(super) fn backend(key: &str, item: &Value) -> Vec<Piece> {
    let tables: Vec<&str> = item["tables"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();

    vec![
        Piece::title("BACKEND", "legacy iptables"),
        Piece::Blank,
        Piece::field(
            "tables",
            match tables.is_empty() {
                true => "none named".to_string(),
                false => tables.join(", "),
            },
        ),
        Piece::field("read from", "/proc/net/ip_tables_names"),
        Piece::field("object", key),
        Piece::Blank,
        Piece::heading("WHAT THIS MEANS"),
        Piece::warning(
            "The nftables ruleset on this host is empty and the legacy iptables backend has \
             tables registered with the kernel. The rules are there and this build does not \
             parse them: it reads the marker and stops. The agent never calls a host in this \
             state a host without a firewall.",
        ),
        Piece::Blank,
    ]
}

fn number(item: &Value, field: &str) -> String {
    item[field]
        .as_i64()
        .map(|count| count.to_string())
        .unwrap_or_else(|| "—".to_string())
}
