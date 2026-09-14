use std::ops::Bound;

use serde_json::Value;

use vigil_model::Snapshot;

use crate::types::Kind;

pub fn objects(reading: &Snapshot, kind: Kind) -> impl Iterator<Item = (&str, &Value)> {
    let prefix = kind.prefix();
    reading
        .items
        .range::<str, _>((Bound::Included(prefix), Bound::Unbounded))
        .take_while(move |(key, _)| key.starts_with(prefix))
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

pub fn flag(item: &Value, field: &str) -> bool {
    item.get(field).and_then(Value::as_bool) == Some(true)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::fixture::users;

    #[test]
    fn the_objects_of_a_kind_are_exactly_the_ones_a_walk_over_the_whole_reading_finds() {
        let mut reading = users();
        reading.items.insert(
            "keyring|root|0x1234".into(),
            json!({"from": "a newer agent"}),
        );
        reading
            .items
            .insert("accounting|x".into(), json!({"looks like": "account"}));

        for kind in [
            Kind::Account,
            Kind::Group,
            Kind::Sudoer,
            Kind::Key,
            Kind::Session,
            Kind::SessionSource,
            Kind::Unknown,
        ] {
            let walked: Vec<&str> = reading
                .items
                .keys()
                .filter(|key| Kind::of(key) == kind)
                .map(String::as_str)
                .collect();
            let found: Vec<&str> = objects(&reading, kind).map(|(key, _)| key).collect();

            assert_eq!(
                found, walked,
                "{kind:?}: the objects of one kind are looked up by the front of their key so a \
                 screen of ten thousand rows does not walk them all for every cell, and the \
                 lookup must find exactly what the walk found"
            );
        }
    }
}
