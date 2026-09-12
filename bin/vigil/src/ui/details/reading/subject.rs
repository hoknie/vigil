use serde_json::Value;

pub struct Subject<'a> {
    pub key: String,
    pub kind: &'static str,
    pub named: String,
    pub means: Vec<String>,
    pub item: &'a Value,
}
