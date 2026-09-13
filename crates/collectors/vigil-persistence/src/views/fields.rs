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

pub(super) fn strings<'a>(item: &'a Value, field: &str) -> Vec<&'a str> {
    item.get(field)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter_map(Value::as_str)
        .collect()
}

pub(super) fn right(said: &str, width: usize) -> String {
    let room = width.saturating_sub(said.chars().count());
    format!("{}{said}", " ".repeat(room))
}

pub(super) fn size_of(bytes: u64) -> String {
    const STEPS: &[(u64, &str)] = &[
        (1024 * 1024 * 1024, "GB"),
        (1024 * 1024, "MB"),
        (1024, "kB"),
    ];

    for (step, name) in STEPS {
        if bytes >= *step {
            return format!("{:.1} {name}", bytes as f64 / *step as f64);
        }
    }

    format!("{bytes} B")
}
