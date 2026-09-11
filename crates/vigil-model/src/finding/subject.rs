use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    #[serde(rename = "type")]
    pub object: String,
    pub key: Value,
}
