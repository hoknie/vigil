use serde_json::{Value, json};
use vigil_model::Snapshot;

use crate::fixture::persistence;

const UNIT: &str = "unit|";

const NAMED: &[&str] = &[
    "wanted_by",
    "required_by",
    "part_of",
    "wants",
    "requires",
    "pulled_in_by",
];

pub(super) fn pulled_in_twice() -> Snapshot {
    let mut reading = persistence();
    reading.items.insert(
        "unit|web.target".to_string(),
        json!({"name": "web.target", "type": "target", "readable": true, "pulled_in_by": []}),
    );
    reading
}

pub(super) fn many_things_started(copies: usize) -> Snapshot {
    let mut reading = pulled_in_twice();
    let read = reading.items.clone();
    for copy in 1..copies {
        let suffix = format!("~{copy:03}");
        for (key, item) in &read {
            let mut item = item.clone();
            if key.starts_with(UNIT) {
                renamed(&mut item, &suffix);
            }
            reading.items.insert(format!("{key}{suffix}"), item);
        }
    }
    reading
}

fn renamed(item: &mut Value, suffix: &str) {
    if let Some(name) = item["name"].as_str() {
        item["name"] = json!(format!("{name}{suffix}"));
    }
    for field in NAMED {
        let Some(names) = item.get_mut(*field).and_then(Value::as_array_mut) else {
            continue;
        };
        for name in names.iter_mut() {
            if let Some(text) = name.as_str() {
                *name = json!(format!("{text}{suffix}"));
            }
        }
    }
}
