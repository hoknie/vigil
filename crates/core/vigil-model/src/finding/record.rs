use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{Evidence, Kind, Rfc3339, Severity, State, Subject, Uuid7};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub event_id: Uuid7,
    pub finding_key: String,
    pub kind: Kind,
    pub severity: Severity,
    pub state: State,
    pub observed_at: Rfc3339,
    pub first_seen_at: Rfc3339,
    pub occurrences: u64,
    pub title: String,
    pub subject: Subject,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub redacted: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}
