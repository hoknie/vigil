use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::Rfc3339;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub source: String,
    pub taken_at: Rfc3339,
    #[serde(default)]
    pub items: BTreeMap<String, Value>,
}

impl Snapshot {
    pub fn new(source: impl Into<String>, taken_at: impl Into<Rfc3339>) -> Self {
        Snapshot {
            source: source.into(),
            taken_at: taken_at.into(),
            items: BTreeMap::new(),
        }
    }

    pub fn with(mut self, key: impl Into<String>, value: Value) -> Self {
        self.items.insert(key.into(), value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_that_names_no_items_at_all_is_a_reading_of_an_empty_host() {
        let from_an_agent_that_left_the_map_out =
            r#"{"source": "ports", "taken_at": "2026-09-09T09:00:00.000Z"}"#;

        let snapshot: Snapshot =
            serde_json::from_str(from_an_agent_that_left_the_map_out).expect("reads");

        assert!(snapshot.items.is_empty());
        assert_eq!(snapshot.source, "ports");
    }
}
