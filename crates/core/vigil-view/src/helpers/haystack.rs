use serde_json::Value;

pub fn haystack(key: &str, item: &Value) -> String {
    let mut text = String::from(key);
    walk(item, &mut text);
    text
}

fn walk(value: &Value, into: &mut String) {
    match value {
        Value::String(text) => {
            into.push(' ');
            into.push_str(text);
        }
        Value::Object(fields) => {
            for field in fields.values() {
                walk(field, into);
            }
        }
        Value::Array(items) => {
            for item in items {
                walk(item, into);
            }
        }
        Value::Null => {}
        other => {
            into.push(' ');
            into.push_str(&other.to_string());
        }
    }
}
