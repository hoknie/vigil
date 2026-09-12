use serde_json::Value;

use super::kind::Kind;

pub struct Row<'a> {
    pub key: String,
    pub kind: Kind,
    pub item: &'a Value,
}
