use serde_json::{Value, json};
use vigil_model::Snapshot;

use crate::fixture::resources;

pub(super) fn many_filesystems(copies: usize) -> Snapshot {
    let mut reading = resources();
    let read = reading.items.clone();
    for copy in 1..copies {
        for (key, item) in &read {
            let mut item = item.clone();
            if let Some(fields) = item.as_object_mut() {
                match copy % 4 {
                    0 => {
                        fields.remove("free_percent_step");
                    }
                    1 => {
                        fields.insert("free_percent_step".into(), json!(copy % 10 * 10));
                    }
                    2 => {
                        fields.insert("free_inodes_percent_step".into(), Value::Null);
                    }
                    _ => {
                        fields.insert("total_bytes".into(), json!(copy % 6 * 4096));
                    }
                }
            }
            reading.items.insert(format!("{key}~{copy:03}"), item);
        }
    }
    reading
}
