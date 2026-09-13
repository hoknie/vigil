use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Os, Peer, Uuid7};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub host_id: String,
    pub install_id: Uuid7,
    pub boot_id: String,
    pub hostname: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    pub os: Os,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tags: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub peer: Option<Peer>,
}
