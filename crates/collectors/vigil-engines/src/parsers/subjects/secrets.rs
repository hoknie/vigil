use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{labels, said, text};
use crate::types::Subject;

const NAME: &[&str] = &["Name", "name", "Spec.Name"];

const DRIVER: &[&str] = &["Driver", "Spec.Driver.Name", "driver"];

pub fn secrets(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read = BTreeMap::new();

    for row in rows {
        let Some(name) = text(row, NAME) else {
            continue;
        };
        let written = labels(row, &["Labels", "Spec.Labels", "labels"]);

        read.insert(
            name.clone(),
            json!({
                "subject": Subject::Secret.as_str(),
                "name": name,
                "id": said(text(row, &["ID", "Id", "id"])),
                "driver": said(text(row, DRIVER)),
                "created_at": said(text(row, &["CreatedAt", "createdAt"])),
                "updated_at": said(text(row, &["UpdatedAt", "updatedAt"])),
                "value_redacted": true,
                "labels": written.named,
                "labels_redacted": written.redacted,
            }),
        );
    }

    read
}
