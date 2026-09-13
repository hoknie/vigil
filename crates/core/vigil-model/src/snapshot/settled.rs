use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettledValue {
    pub value: Value,
    pub from: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settled {
    pub source: String,
    pub values: BTreeMap<String, BTreeMap<String, SettledValue>>,
}

impl Settled {
    pub fn new(source: impl Into<String>) -> Settled {
        Settled {
            source: source.into(),
            values: BTreeMap::new(),
        }
    }

    pub fn pinning(
        mut self,
        key: impl Into<String>,
        path: impl Into<String>,
        value: impl Into<Value>,
        from: impl Into<String>,
    ) -> Settled {
        self.values.entry(key.into()).or_default().insert(
            path.into(),
            SettledValue {
                value: value.into(),
                from: from.into(),
            },
        );
        self
    }

    pub fn written(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("a pinned value is plain data");
        text.push('\n');
        text
    }

    pub fn read(text: &str) -> Result<Settled, String> {
        serde_json::from_str(text).map_err(|error| error.to_string())
    }

    pub fn missed(&self, items: &BTreeMap<String, Value>) -> Vec<String> {
        let mut missed = Vec::new();

        for (key, pinned) in &self.values {
            match items.get(key) {
                Some(item) => missed.extend(self.missed_in(key, item)),
                None => missed.extend(pinned.iter().map(|(path, settled)| {
                    format!(
                        "{key} is a row this side never carries, and its {path} is {}: {}",
                        settled.value, settled.from
                    )
                })),
            }
        }

        missed
    }

    pub fn missed_in(&self, key: &str, item: &Value) -> Vec<String> {
        let Some(pinned) = self.values.get(key) else {
            return Vec::new();
        };

        let mut missed = Vec::new();

        for (path, settled) in pinned {
            let right = &settled.value;
            let from = &settled.from;

            match value_at(item, path) {
                Some(found) if found == right => {}
                Some(found) => missed.push(format!(
                    "{key} says {path} is {found}, and it is {right}: {from}"
                )),
                None => missed.push(format!(
                    "{key} says nothing about {path}, and it is {right}: {from}"
                )),
            }
        }

        missed
    }
}

fn value_at<'a>(item: &'a Value, path: &str) -> Option<&'a Value> {
    let mut here = item;

    for segment in path.split('/') {
        here = here.as_object()?.get(segment)?;
    }

    Some(here)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn answer(items: &[(&str, Value)]) -> BTreeMap<String, Value> {
        items
            .iter()
            .map(|(key, value)| ((*key).to_string(), value.clone()))
            .collect()
    }

    fn periods() -> Settled {
        Settled::new("status").pinning(
            "collector-ok|ports",
            "every_seconds",
            30,
            "the period the collector declares for ports",
        )
    }

    #[test]
    fn an_answer_that_carries_every_settled_value_complains_about_nothing() {
        let missed = periods().missed(&answer(&[(
            "collector-ok|ports",
            json!({"name": "ports", "every_seconds": 30}),
        )]));

        assert!(missed.is_empty(), "{missed:?}");
    }

    #[test]
    fn a_value_that_drifted_names_the_right_one_and_where_it_comes_from() {
        let missed = periods().missed(&answer(&[(
            "collector-ok|ports",
            json!({"name": "ports", "every_seconds": 15}),
        )]));

        assert_eq!(missed.len(), 1, "{missed:?}");
        assert!(missed[0].contains("is 15"), "{}", missed[0]);
        assert!(missed[0].contains("it is 30"), "{}", missed[0]);
        assert!(
            missed[0].contains("the period the collector declares for ports"),
            "a value named without its source is a value nobody knows where to correct: {}",
            missed[0]
        );
    }

    #[test]
    fn a_field_the_row_does_not_carry_at_all_is_named_rather_than_passed() {
        let missed = periods().missed(&answer(&[("collector-ok|ports", json!({"name": "ports"}))]));

        assert_eq!(missed.len(), 1, "{missed:?}");
        assert!(missed[0].contains("says nothing about"), "{}", missed[0]);
    }

    #[test]
    fn a_row_the_sample_pins_and_this_side_never_sends_is_named() {
        let missed = periods().missed(&answer(&[("collector-off|launches", json!({}))]));

        assert_eq!(missed.len(), 1, "{missed:?}");
        assert!(missed[0].contains("never carries"), "{}", missed[0]);
    }

    #[test]
    fn a_pin_reaches_a_field_inside_an_object() {
        let pinned = Settled::new("status").pinning(
            "agent|watching",
            "findings/capacity",
            500,
            "the number of findings the daemon holds for the console",
        );

        assert!(
            pinned
                .missed(&answer(&[(
                    "agent|watching",
                    json!({"findings": {"capacity": 500}})
                )]))
                .is_empty()
        );
        assert_eq!(
            pinned
                .missed(&answer(&[(
                    "agent|watching",
                    json!({"findings": {"capacity": 200}})
                )]))
                .len(),
            1
        );
    }

    #[test]
    fn a_field_the_daemon_leaves_empty_is_answered_by_nothing_and_not_by_a_number() {
        let pinned = Settled::new("status").pinning(
            "collector-off|launches",
            "every_seconds",
            Value::Null,
            "a collector nothing asks for is read on no period",
        );

        assert!(
            pinned
                .missed(&answer(&[(
                    "collector-off|launches",
                    json!({"every_seconds": null})
                )]))
                .is_empty()
        );
        assert_eq!(
            pinned
                .missed(&answer(&[(
                    "collector-off|launches",
                    json!({"every_seconds": 15})
                )]))
                .len(),
            1,
            "a period on a collector nobody runs is a screen promising a reading that never comes"
        );
    }

    #[test]
    fn a_pin_into_a_list_is_a_pin_this_algebra_cannot_answer() {
        let pinned = Settled::new("status").pinning(
            "agent|watching",
            "collectors/every_seconds",
            30,
            "the period the collector declares for ports",
        );

        let missed = pinned.missed(&answer(&[(
            "agent|watching",
            json!({"collectors": [{"every_seconds": 30}]}),
        )]));

        assert_eq!(
            missed.len(),
            1,
            "a value inside a list is pinned on the row of its own, not through the list"
        );
    }

    #[test]
    fn a_row_the_sample_pins_nothing_in_is_a_row_this_holds_nothing_against() {
        assert!(
            periods()
                .missed_in("collector-ok|users", &json!({"every_seconds": 1}))
                .is_empty(),
            "a row nobody pinned is a row this algebra says nothing about; what the sample has to \
             carry is the answer to a caller that asks for every row"
        );
    }

    #[test]
    fn the_same_pins_are_written_the_same_way_twice() {
        let first = Settled::new("status")
            .pinning("collector-ok|ports", "name", "ports", "the collector")
            .pinning("collector-ok|ports", "every_seconds", 30, "the collector");
        let again = Settled::new("status")
            .pinning("collector-ok|ports", "every_seconds", 30, "the collector")
            .pinning("collector-ok|ports", "name", "ports", "the collector");

        assert_eq!(first.written(), again.written());
        assert!(first.written().ends_with("}\n"));
        assert_eq!(Settled::read(&first.written()).expect("reads"), first);
    }
}
