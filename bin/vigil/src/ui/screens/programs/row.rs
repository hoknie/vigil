use serde_json::Value;

pub struct Row<'a> {
    pub key: String,
    pub item: &'a Value,
    pub mark: bool,
}
