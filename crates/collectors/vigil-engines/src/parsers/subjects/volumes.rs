use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{PROJECT, labels, of, said, text};
use crate::types::Subject;

const DEVICE: &[&str] = &["Options.device", "options.device"];

pub fn volumes(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read = BTreeMap::new();

    for row in rows {
        let Some(name) = text(row, &["Name", "name"]) else {
            continue;
        };
        let written = labels(row, &["Labels", "labels"]);

        read.insert(
            name.clone(),
            json!({
                "subject": Subject::Volume.as_str(),
                "name": name,
                "driver": said(text(row, &["Driver", "driver"])),
                "mountpoint": said(text(row, &["Mountpoint", "mountpoint"])),
                "device": said(text(row, DEVICE)),
                "scope": said(text(row, &["Scope", "scope"])),
                "labels": written.named,
                "labels_redacted": written.redacted,
                "project": said(of(&written, PROJECT)),
            }),
        );
    }

    read
}
