use vigil_model::Shape;

const ABSENT: &str = "null";

pub fn outside_the_sample(reading: &Shape, sample: &Shape, absent: &[&str]) -> Vec<String> {
    let mut drift = Vec::new();

    if reading.source != sample.source {
        drift.push(format!(
            "the reading is of {} and the sample of {}",
            reading.source, sample.source
        ));
    }

    for (class, fields) in &reading.classes {
        let Some(known) = sample.classes.get(class) else {
            drift.push(format!("{class}: a class of row the sample does not have"));
            continue;
        };
        for (path, field) in fields {
            let Some(expected) = known.get(path) else {
                drift.push(format!("{class}.{path}: a field the sample does not have"));
                continue;
            };
            let may_be_absent = absent.contains(&format!("{class}.{path}").as_str());
            let foreign: Vec<&String> = field
                .types
                .difference(&expected.types)
                .filter(|kind| !(may_be_absent && kind.as_str() == ABSENT))
                .collect();
            if !foreign.is_empty() {
                drift.push(format!(
                    "{class}.{path}: holds {foreign:?}, where the sample holds {:?}",
                    expected.types
                ));
            }
        }
        for (path, expected) in known {
            let always = fields.get(path).is_some_and(|field| field.always);
            if expected.always && !always {
                drift.push(format!(
                    "{class}.{path}: the sample has it on every row and this reading does not"
                ));
            }
        }
    }

    drift
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use vigil_model::Snapshot;

    use super::*;

    fn shape(items: &[(&str, serde_json::Value)]) -> Shape {
        let mut snapshot = Snapshot::new("network", "2026-09-19T12:00:00.000Z");
        for (key, item) in items {
            snapshot.items.insert((*key).to_string(), item.clone());
        }
        Shape::of(&snapshot)
    }

    #[test]
    fn a_reading_of_the_same_rows_with_fewer_of_them_is_inside_the_sample() {
        let sample = shape(&[
            ("tcp|0.0.0.0:22", json!({"port": 22, "user": "root"})),
            ("tcp|0.0.0.0:80", json!({"port": 80, "user": null})),
            ("unix|/run/a", json!({"path": "/run/a"})),
        ]);
        let reading = shape(&[("tcp|0.0.0.0:22", json!({"port": 22, "user": "root"}))]);

        assert_eq!(
            outside_the_sample(&reading, &sample, &[]),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_field_the_sample_never_had_and_a_type_it_never_held_are_both_named() {
        let sample = shape(&[("tcp|0.0.0.0:22", json!({"port": 22}))]);
        let reading = shape(&[("tcp|0.0.0.0:22", json!({"port": "22", "family": "inet"}))]);

        let drift = outside_the_sample(&reading, &sample, &[]);

        assert_eq!(drift.len(), 2, "{drift:?}");
        assert!(drift.iter().any(|said| said.contains("tcp.family")));
        assert!(drift.iter().any(|said| said.contains("tcp.port")));
    }

    #[test]
    fn a_field_every_row_of_the_sample_carries_is_missed_when_a_row_of_the_reading_lacks_it() {
        let sample = shape(&[("tcp|0.0.0.0:22", json!({"port": 22, "uid": 0}))]);
        let reading = shape(&[("tcp|0.0.0.0:22", json!({"port": 22}))]);

        assert_eq!(
            outside_the_sample(&reading, &sample, &[]),
            vec!["tcp.uid: the sample has it on every row and this reading does not".to_string()],
            "a rule reads a field it was promised, and a reading of another system that leaves \
             it out is a rule that silently reads nothing"
        );
    }

    #[test]
    fn a_class_of_row_the_sample_does_not_have_is_named() {
        let sample = shape(&[("tcp|0.0.0.0:22", json!({"port": 22}))]);
        let reading = shape(&[("sctp|0.0.0.0:22", json!({"port": 22}))]);

        assert_eq!(outside_the_sample(&reading, &sample, &[]).len(), 1);
    }

    #[test]
    fn a_field_named_as_absent_on_this_system_may_be_null_where_the_sample_holds_a_value() {
        let sample = shape(&[(
            "memory|summary",
            json!({"total_bytes": 8, "swap_total_bytes": 4}),
        )]);
        let reading = shape(&[(
            "memory|summary",
            json!({"total_bytes": 8, "swap_total_bytes": null}),
        )]);

        assert_eq!(
            outside_the_sample(&reading, &sample, &["memory.swap_total_bytes"]),
            Vec::<String>::new()
        );
        assert_eq!(
            outside_the_sample(&reading, &sample, &[]).len(),
            1,
            "a value another system has and this one does not is left out on purpose, and              only a field named as such is let through"
        );
        let wrong = shape(&[(
            "memory|summary",
            json!({"total_bytes": 8, "swap_total_bytes": "4"}),
        )]);
        assert_eq!(
            outside_the_sample(&wrong, &sample, &["memory.swap_total_bytes"]).len(),
            1
        );
    }
}
