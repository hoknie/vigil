use serde::{Deserialize, Serialize};

use crate::{Finding, Host, Rfc3339, SchemaVersion, Uuid7};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub schema_version: SchemaVersion,
    pub batch_id: Uuid7,
    pub sent_at: Rfc3339,
    pub producer: Producer,
    pub host: Host,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Producer {
    pub name: String,
    pub version: String,
}
