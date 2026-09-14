use std::collections::BTreeMap;

use vigil_model::{Host, Rfc3339};

pub struct Startup {
    pub host: Host,
    pub configuration_path: String,
    pub started_at: Rfc3339,
    pub interval_seconds: u32,
    pub periods: BTreeMap<String, u32>,
    pub killing_from_the_console: bool,
}
