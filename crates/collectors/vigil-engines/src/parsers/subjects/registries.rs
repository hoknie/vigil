use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::types::{Registry, Subject};

pub fn registries(found: &[Registry]) -> BTreeMap<String, Value> {
    found
        .iter()
        .map(|registry| {
            (
                registry.host.clone(),
                json!({
                    "subject": Subject::Registry.as_str(),
                    "host": registry.host,
                    "insecure": registry.insecure,
                    "role": registry.role,
                    "from": registry.from,
                }),
            )
        })
        .collect()
}
