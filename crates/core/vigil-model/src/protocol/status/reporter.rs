use serde::{Deserialize, Serialize};

use crate::Rfc3339;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterStatus {
    pub name: String,
    pub deliveries: u64,
    pub failures: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sent_at: Option<Rfc3339>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}
