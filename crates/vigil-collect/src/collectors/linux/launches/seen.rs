use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::Value;

#[derive(Default)]
pub(super) struct Seen {
    pub(super) offset: u64,
    pub(super) inode: u64,
    pub(super) from: Option<PathBuf>,
    pub(super) items: BTreeMap<String, Value>,
    pub(super) rule_loaded: bool,
}
