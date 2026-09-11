use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::reading::Snapshot;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapeField {
    pub types: BTreeSet<String>,
    pub always: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shape {
    pub source: String,
    pub classes: BTreeMap<String, BTreeMap<String, ShapeField>>,
}

impl Shape {
    pub fn of(snapshot: &Snapshot) -> Shape {
        Shape::of_items(&snapshot.source, &snapshot.items)
    }

    pub fn of_items(source: &str, items: &BTreeMap<String, Value>) -> Shape {
        let mut counted: BTreeMap<String, usize> = BTreeMap::new();
        let mut gathered: BTreeMap<String, BTreeMap<String, Gathered>> = BTreeMap::new();

        for (key, item) in items {
            let class = class_of(key).to_string();
            *counted.entry(class.clone()).or_default() += 1;
            let held = gathered.entry(class).or_default();

            for (path, types) in paths_of(item) {
                let field = held.entry(path).or_default();
                field.types.extend(types);
                field.items += 1;
            }
        }

        Shape {
            source: source.to_string(),
            classes: gathered
                .into_iter()
                .map(|(class, fields)| {
                    let items = counted.get(&class).copied().unwrap_or_default();
                    let fields = fields
                        .into_iter()
                        .map(|(path, field)| {
                            (
                                path,
                                ShapeField {
                                    types: field.types,
                                    always: field.items == items,
                                },
                            )
                        })
                        .collect();
                    (class, fields)
                })
                .collect(),
        }
    }

    pub fn written(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("a shape is plain data");
        text.push('\n');
        text
    }
}

#[derive(Default)]
struct Gathered {
    types: BTreeSet<String>,
    items: usize,
}

pub fn class_of(key: &str) -> &str {
    key.split('|').next().unwrap_or_default()
}

fn paths_of(item: &Value) -> BTreeMap<String, BTreeSet<String>> {
    let mut found = BTreeMap::new();

    if let Value::Object(fields) = item {
        for (name, value) in fields {
            walk(name.clone(), value, &mut found);
        }
    }

    found
}

fn walk(path: String, value: &Value, found: &mut BTreeMap<String, BTreeSet<String>>) {
    found
        .entry(path.clone())
        .or_default()
        .insert(type_of(value).to_string());

    match value {
        Value::Object(fields) => {
            for (name, inner) in fields {
                walk(format!("{path}/{name}"), inner, found);
            }
        }
        Value::Array(items) => {
            for inner in items {
                walk(format!("{path}/[]"), inner, found);
            }
        }
        _ => {}
    }
}

fn type_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn reading(items: &[(&str, Value)]) -> Snapshot {
        let mut snapshot = Snapshot::new("persistence", "2026-09-11T09:00:00.000Z");
        for (key, value) in items {
            snapshot.items.insert((*key).to_string(), value.clone());
        }
        snapshot
    }

    fn field<'a>(shape: &'a Shape, class: &str, path: &str) -> &'a ShapeField {
        shape.classes[class]
            .get(path)
            .unwrap_or_else(|| panic!("{class} has no {path}: {:?}", shape.classes[class].keys()))
    }

    #[test]
    fn a_list_of_one_word_is_a_list_and_not_the_word() {
        let listed = Shape::of(&reading(&[(
            "timer|one",
            json!({"on_calendar": ["daily"]}),
        )]));
        let spoken = Shape::of(&reading(&[("timer|one", json!({"on_calendar": "daily"}))]));

        assert_eq!(
            field(&listed, "timer", "on_calendar").types,
            BTreeSet::from(["array".to_string()])
        );
        assert_eq!(
            field(&listed, "timer", "on_calendar/[]").types,
            BTreeSet::from(["string".to_string()])
        );
        assert_ne!(listed, spoken);
    }

    #[test]
    fn a_field_missing_from_one_item_is_not_required_of_the_class() {
        let shape = Shape::of(&reading(&[
            ("unit|a", json!({"name": "a", "wants": ["b"]})),
            ("unit|b", json!({"name": "b"})),
        ]));

        assert!(field(&shape, "unit", "name").always);
        assert!(!field(&shape, "unit", "wants").always);
    }

    #[test]
    fn a_null_carries_no_fields_underneath_it() {
        let shape = Shape::of(&reading(&[
            ("tcp|a", json!({"process": {"exe": "/usr/sbin/nginx"}})),
            ("tcp|b", json!({"process": null})),
        ]));

        assert_eq!(
            field(&shape, "tcp", "process").types,
            BTreeSet::from(["null".to_string(), "object".to_string()])
        );
        assert!(
            !field(&shape, "tcp", "process/exe").always,
            "the item whose owner did not resolve carries no path under it"
        );
    }

    #[test]
    fn an_empty_list_does_not_claim_to_know_its_elements() {
        let shape = Shape::of(&reading(&[("unit|a", json!({"commands": []}))]));

        assert!(shape.classes["unit"].contains_key("commands"));
        assert!(
            !shape.classes["unit"].contains_key("commands/[]"),
            "nobody has seen an element, and saying nothing about it is the honest answer"
        );
    }

    #[test]
    fn the_same_reading_is_written_the_same_way_twice() {
        let first = Shape::of(&reading(&[
            ("unit|b", json!({"name": "b", "wants": ["x", "a"]})),
            ("unit|a", json!({"name": "a", "readable": true})),
        ]));
        let again = Shape::of(&reading(&[
            ("unit|a", json!({"readable": true, "name": "a"})),
            ("unit|b", json!({"wants": ["a", "x"], "name": "b"})),
        ]));

        assert_eq!(first.written(), again.written());
        assert!(first.written().ends_with("}\n"));
    }

    #[test]
    fn a_marker_row_is_a_class_of_its_own_when_its_key_says_so() {
        let shape = Shape::of(&reading(&[
            ("module|overlay", json!({"name": "overlay"})),
            ("modules|unreadable", json!({"readable": false})),
        ]));

        assert!(shape.classes.contains_key("module"));
        assert!(
            shape.classes.contains_key("modules"),
            "a note about what could not be read is not a kernel module"
        );
    }
}
