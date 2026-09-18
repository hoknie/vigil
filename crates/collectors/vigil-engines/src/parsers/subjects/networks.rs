use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{PROJECT, at, flag, labels, of, said, text, words};
use crate::types::Subject;

const SUBNETS: &[&str] = &["subnets", "Subnets"];

pub fn networks(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read = BTreeMap::new();

    for row in rows {
        let Some(name) = text(row, &["Name", "name"]) else {
            continue;
        };
        let written = labels(row, &["Labels", "labels"]);

        read.insert(
            name.clone(),
            json!({
                "subject": Subject::Network.as_str(),
                "name": name,
                "id": said(text(row, &["ID", "Id", "id"])),
                "driver": said(text(row, &["Driver", "driver"])),
                "interface": said(text(row, &["network_interface"])),
                "internal": flag(row, &["Internal", "internal"]).unwrap_or(false),
                "ipv6": flag(row, &["IPv6", "ipv6_enabled"]).unwrap_or(false),
                "subnets": subnets_of(row),
                "labels": written.named,
                "labels_redacted": written.redacted,
                "project": said(of(&written, PROJECT)),
            }),
        );
    }

    read
}

fn subnets_of(row: &Value) -> Vec<String> {
    let Some(Value::Array(listed)) = SUBNETS.iter().find_map(|spelling| at(row, spelling)) else {
        return words(row, SUBNETS);
    };

    let mut read: Vec<String> = listed
        .iter()
        .filter_map(|one| match one {
            Value::String(said) => Some(said.clone()),
            other => text(other, &["subnet", "Subnet"]),
        })
        .collect();
    read.sort_unstable();
    read.dedup();
    read
}
