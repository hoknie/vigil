use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindingsSummary {
    pub retained: usize,
    pub capacity: usize,
    pub total: u64,
    pub dropped: u64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub by_severity: BTreeMap<String, u64>,
}
