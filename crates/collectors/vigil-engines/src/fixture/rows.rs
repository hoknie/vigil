use serde_json::Value;
use vigil_model::Snapshot;

use crate::parsers::key;
use crate::types::{Engine, Subject};

pub const UNTAGGED_IMAGE: &str = "sha256:5f0c1ad8b29e";

pub const TAGGED_IMAGE: &str = "sha256:18ad9bdc4c87";

pub const HOST_NETWORK_CONTAINER: &str = "metrics-agent-1";

pub const CONTAINER_MOUNTING_ETC: &str = "shop-web-1";

pub const VOLUME_FROM_ETC: &str = "etc_backup";

pub const BRIDGE_WITH_A_SUBNET: &str = "podman1";

pub const INSECURE_REGISTRY: &str = "registry.local:5000";

pub const POD: &str = "tools";

pub const SECRET: &str = "shop-database-password";

pub const PROJECTS: [&str; 2] = ["shop", "metrics"];

pub fn row(reading: &Snapshot, engine: Engine, subject: Subject, id: &str) -> Value {
    let wanted = key(engine, subject, id);

    reading
        .items
        .get(&wanted)
        .cloned()
        .unwrap_or_else(|| panic!("{wanted} is not in the sample reading of the engines"))
}
