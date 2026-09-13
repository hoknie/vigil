use serde_json::Value;

pub(super) fn text<'a>(item: &'a Value, field: &str) -> Option<&'a str> {
    item.get(field).and_then(Value::as_str)
}

pub(super) fn number(item: &Value, field: &str) -> Option<u64> {
    item.get(field).and_then(Value::as_u64)
}

pub(super) fn flag(item: &Value, field: &str) -> bool {
    item.get(field).and_then(Value::as_bool) == Some(true)
}

pub(super) fn marked(key: &str) -> bool {
    key.starts_with("launches|")
}
