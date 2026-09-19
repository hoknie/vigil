use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Default)]
pub(super) struct Seen {
    pub(super) offset: u64,
    pub(super) inode: u64,
    pub(super) items: BTreeMap<String, Value>,
}
