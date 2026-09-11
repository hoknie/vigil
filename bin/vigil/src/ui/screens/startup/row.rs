use serde_json::Value;

use super::kind::Kind;

pub struct Row<'a> {
    pub key: String,
    pub kind: Kind,
    pub item: &'a Value,
    pub depth: usize,
    pub parents: usize,
}

impl Row<'_> {
    pub fn mark(&self) -> bool {
        self.kind.mark()
    }
}
