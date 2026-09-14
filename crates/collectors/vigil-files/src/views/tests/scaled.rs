use serde_json::json;
use vigil_model::Snapshot;

use crate::fixture::files;

pub(super) fn many_files(copies: usize) -> Snapshot {
    let mut reading = files();
    let read = reading.items.clone();
    for copy in 1..copies {
        for (key, item) in &read {
            let mut item = item.clone();
            if copy % 3 == 0 {
                item["mode"] = json!(format!("07{}{}", copy % 8, copy % 5));
            }
            if copy % 4 == 1 {
                item["size"] = json!(copy % 7);
            }
            if copy % 5 == 2 {
                item["uid"] = json!(copy % 3);
            }
            reading.items.insert(format!("{key}~{copy:03}"), item);
        }
    }
    reading
}
