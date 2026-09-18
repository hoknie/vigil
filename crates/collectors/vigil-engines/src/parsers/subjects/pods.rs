use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{at, labels, said, text, words};
use crate::types::Subject;

pub fn pods(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read = BTreeMap::new();

    for row in rows {
        let Some(name) = text(row, &["Name", "name"]) else {
            continue;
        };
        let written = labels(row, &["Labels", "labels"]);

        read.insert(
            name.clone(),
            json!({
                "subject": Subject::Pod.as_str(),
                "name": name,
                "id": said(text(row, &["Id", "ID", "id"])),
                "infra_id": said(text(row, &["InfraId", "InfraID"])),
                "cgroup": said(text(row, &["Cgroup", "cgroup"])),
                "containers": held(row),
                "networks": words(row, &["Networks", "networks"]),
                "labels": written.named,
                "labels_redacted": written.redacted,
            }),
        );
    }

    read
}

fn held(row: &Value) -> Vec<String> {
    let Some(Value::Array(listed)) = at(row, "Containers") else {
        return words(row, &["Containers", "containers"]);
    };

    let mut read: Vec<String> = listed
        .iter()
        .filter_map(|one| match one {
            Value::String(said) => Some(said.clone()),
            other => text(other, &["Names", "Name", "names"]),
        })
        .collect();
    read.sort_unstable();
    read.dedup();
    read
}
