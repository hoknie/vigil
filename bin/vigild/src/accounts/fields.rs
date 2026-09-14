use serde_json::Value;
use vigil_model::Snapshot;

pub const SUDOERS_D: &str = "/etc/sudoers.d/";

pub fn item<'a>(reading: &'a Snapshot, key: &str) -> Option<&'a Value> {
    reading.items.get(key)
}

pub fn number(item: &Value, field: &str) -> Option<u64> {
    item.get(field).and_then(Value::as_u64)
}

pub fn text<'a>(item: &'a Value, field: &str) -> Option<&'a str> {
    item.get(field).and_then(Value::as_str)
}

pub fn flag(item: &Value, field: &str) -> bool {
    item.get(field).and_then(Value::as_bool) == Some(true)
}

pub fn key_rows<'a>(reading: &'a Snapshot, user: &str) -> Vec<(&'a str, &'a Value)> {
    let prefix = format!("sshkey|{user}|");
    reading
        .items
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(key, item)| (key.as_str(), item))
        .collect()
}

pub fn sources<'a>(rows: &[(&'a str, &'a Value)]) -> Vec<&'a str> {
    let mut named: Vec<&str> = Vec::new();
    for (_, item) in rows {
        if let Some(source) = text(item, "source")
            && !named.contains(&source)
        {
            named.push(source);
        }
    }
    named
}

pub fn sudoers_d_sources(grant: &Value) -> Vec<&str> {
    let mut named: Vec<&str> = Vec::new();
    for rule in grant
        .get("rules")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        if let Some(source) = text(rule, "source")
            && source.starts_with(SUDOERS_D)
            && !named.contains(&source)
        {
            named.push(source);
        }
    }
    named
}
