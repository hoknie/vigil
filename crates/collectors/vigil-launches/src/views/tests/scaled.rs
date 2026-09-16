use serde_json::json;
use vigil_model::Snapshot;

use crate::fixture::launches;

pub(super) fn many_launches(copies: usize) -> Snapshot {
    let mut reading = launches();
    let read = reading.items.clone();
    for copy in 1..copies {
        for (key, item) in &read {
            let mut item = item.clone();
            if let Some(fields) = item.as_object_mut() {
                if copy % 2 == 0 {
                    fields.insert("runs".into(), json!(copy % 13 * 9 + 1));
                }
                if copy % 3 == 0 {
                    fields.remove("first_seen");
                }
                if copy % 3 == 1 {
                    fields.insert(
                        "first_seen".into(),
                        json!(format!("2026-09-{:02}T12:00:00.000Z", copy % 4 + 1)),
                    );
                }
                if copy % 7 == 3 {
                    fields.remove("user");
                }
                match copy % 5 {
                    1 => {
                        fields.insert(
                            "last_audit_id".into(),
                            json!(format!(
                                "{}.{:03}:{}",
                                99_999_990 + copy % 4,
                                copy % 3,
                                copy % 2
                            )),
                        );
                    }
                    2 => {
                        fields.remove("last_audit_id");
                        fields.remove("audit_id");
                    }
                    3 => {
                        fields.insert("last_audit_id".into(), json!("not an audit id"));
                    }
                    _ => {}
                }
            }
            reading.items.insert(format!("{key}~{copy:03}"), item);
        }
    }
    reading
}
