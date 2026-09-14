use serde_json::json;
use vigil_model::Snapshot;

use crate::fixture::processes;

pub(super) fn many_programs(copies: usize) -> Snapshot {
    let mut reading = processes();
    let read = reading.items.clone();
    for copy in 1..copies {
        for (key, item) in &read {
            let mut item = item.clone();
            if copy % 3 == 0 {
                item["user"] = json!(format!("worker{}", copy % 4));
            }
            reading.items.insert(format!("{key}~{copy:03}"), item);
        }
    }
    reading
}
