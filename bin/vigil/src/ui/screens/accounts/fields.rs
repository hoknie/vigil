use serde_json::Value;

use super::kind::Kind;
use crate::ui::{Reading, View};

pub fn objects(view: &View, kind: Kind) -> impl Iterator<Item = (&str, &Value)> {
    let items = match view.reading("users") {
        Reading::Taken(snapshot) => Some(&snapshot.items),
        _ => None,
    };
    items
        .into_iter()
        .flatten()
        .filter(move |(key, _)| Kind::of(key) == kind)
        .map(|(key, item)| (key.as_str(), item))
}

pub fn text<'a>(item: &'a Value, field: &str) -> Option<&'a str> {
    item.get(field).and_then(Value::as_str)
}

pub fn number(item: &Value, field: &str) -> String {
    match item.get(field).and_then(Value::as_u64) {
        Some(value) => value.to_string(),
        None => "?".to_string(),
    }
}

pub fn members(group: &Value) -> impl Iterator<Item = &str> {
    group
        .get("members")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
}

pub fn rules(grant: &Value) -> impl Iterator<Item = &Value> {
    grant
        .get("rules")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
}
