use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{PROJECT, SERVICE, at, labels, of, said, text, words};
use crate::types::Subject;

const CUT_SHORT: char = '…';

const HOST_NETWORK: &str = "host";

pub fn containers(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read = BTreeMap::new();

    for row in rows {
        let Some(name) = name_of(row) else {
            continue;
        };
        let written = labels(row, &["Labels", "labels"]);
        let mounts = words(row, &["Mounts", "mounts"]);
        let networks = words(row, &["Networks", "networks"]);
        let on_the_host_network = networks.iter().any(|one| one == HOST_NETWORK);
        let cut_short = mounts.iter().any(|one| one.contains(CUT_SHORT));

        read.insert(
            name.clone(),
            json!({
                "subject": Subject::Container.as_str(),
                "name": name,
                "id": said(text(row, &["ID", "Id", "id"])),
                "image": said(text(row, &["Image", "image"])),
                "image_id": said(text(row, &["ImageID", "ImageId"])),
                "networks": networks,
                "host_network": on_the_host_network,
                "mounts": mounts,
                "mounts_truncated": cut_short,
                "ports": ports_of(row),
                "pod": said(text(row, &["PodName", "Pod", "pod"])),
                "labels": written.named,
                "labels_redacted": written.redacted,
                "project": said(of(&written, PROJECT)),
                "service": said(of(&written, SERVICE)),
            }),
        );
    }

    read
}

fn name_of(row: &Value) -> Option<String> {
    let listed = words(row, &["Names", "names"]);
    match listed.first() {
        Some(first) => Some(first.clone()),
        None => text(row, &["Name", "name"]),
    }
}

fn ports_of(row: &Value) -> Vec<String> {
    let Some(Value::Array(listed)) = at(row, "Ports") else {
        return words(row, &["Ports", "ports"]);
    };

    let mut read: Vec<String> = listed.iter().filter_map(published).collect();
    read.sort_unstable();
    read.dedup();
    read
}

fn published(one: &Value) -> Option<String> {
    if let Value::String(said) = one {
        return Some(said.clone());
    }

    let container = text(one, &["container_port"])?;
    let protocol = text(one, &["protocol"]).unwrap_or_else(|| "tcp".to_string());
    match (text(one, &["host_ip"]), text(one, &["host_port"])) {
        (_, None) => Some(format!("{container}/{protocol}")),
        (address, Some(host)) => Some(format!(
            "{}:{host}->{container}/{protocol}",
            address.unwrap_or_else(|| "0.0.0.0".to_string())
        )),
    }
}
