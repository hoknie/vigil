use vigil_model::{Change, Snapshot};

pub fn diff(before: &Snapshot, after: &Snapshot) -> Vec<Change> {
    debug_assert_eq!(
        before.source, after.source,
        "diffing two different collectors is a programming error, not a change"
    );

    let mut changes = Vec::new();
    for (key, new_value) in &after.items {
        match before.items.get(key) {
            None => changes.push(Change::Added {
                key: key.clone(),
                after: new_value.clone(),
            }),
            Some(old_value) if old_value != new_value => changes.push(Change::Changed {
                key: key.clone(),
                before: old_value.clone(),
                after: new_value.clone(),
            }),
            Some(_) => {}
        }
    }
    for (key, old_value) in &before.items {
        if !after.items.contains_key(key) {
            changes.push(Change::Removed {
                key: key.clone(),
                before: old_value.clone(),
            });
        }
    }
    changes
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::{Change, Snapshot};

    use super::diff;

    fn ports(items: &[(&str, serde_json::Value)]) -> Snapshot {
        let mut snapshot = Snapshot::new("ports", "2026-09-08T12:00:00.000Z");
        for (key, value) in items {
            snapshot.items.insert((*key).to_string(), value.clone());
        }
        snapshot
    }

    #[test]
    fn reports_new_gone_and_altered_keys_and_nothing_else() {
        let before = ports(&[
            ("tcp|127.0.0.1:5432", json!({"process": "postgres"})),
            ("tcp|0.0.0.0:80", json!({"process": "nginx"})),
        ]);
        let after = ports(&[
            ("tcp|0.0.0.0:80", json!({"process": "nginx"})),
            ("tcp|0.0.0.0:4444", json!({"process": "nc"})),
            ("tcp|0.0.0.0:5432", json!({"process": "postgres"})),
        ]);

        let changes = diff(&before, &after);

        assert_eq!(changes.len(), 3, "the unchanged :80 must not be reported");
        assert!(matches!(&changes[0], Change::Added { key, .. } if key == "tcp|0.0.0.0:4444"));
        assert!(matches!(&changes[1], Change::Added { key, .. } if key == "tcp|0.0.0.0:5432"));
        assert!(matches!(&changes[2], Change::Removed { key, .. } if key == "tcp|127.0.0.1:5432"));
    }

    #[test]
    fn a_value_change_under_the_same_key_is_a_change_not_a_pair() {
        let before = ports(&[("tcp|0.0.0.0:80", json!({"process": "nginx"}))]);
        let after = ports(&[("tcp|0.0.0.0:80", json!({"process": "nc"}))]);

        let changes = diff(&before, &after);

        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], Change::Changed { key, .. } if key == "tcp|0.0.0.0:80"));
    }

    #[test]
    fn the_first_reading_of_a_host_is_not_a_hundred_findings() {
        let empty = ports(&[]);
        let first = ports(&[("tcp|0.0.0.0:80", json!({"process": "nginx"}))]);

        assert_eq!(diff(&empty, &first).len(), 1);
    }

    #[test]
    fn diffing_two_identical_snapshots_of_ten_thousand_items_finds_nothing() {
        let mut before = Snapshot::new("ports", "2026-09-10T12:00:00.000Z");
        for number in 0..10_000u32 {
            before.items.insert(
                format!("tcp|10.0.0.{}:{}", number / 250, 1_024 + number % 250),
                json!({"protocol": "tcp", "port": 1_024 + number % 250, "process": "nginx"}),
            );
        }
        let after = before.clone();

        assert!(
            diff(&before, &after).is_empty(),
            "a host that did not move must cost nothing to talk about"
        );
    }
}
