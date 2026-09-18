use serde_json::Value;
use vigil_view::Piece;

use super::facts::{id_of, warnings};
use super::meanings::meant;
use crate::helpers::{field_list, finding_key};
use crate::types::{Engine, List};

const NOT_PRINTED: &str = "not printed";

pub(super) fn detail(engine: Engine, list: List, key: &str, item: &Value) -> Vec<Piece> {
    let mut said = vec![
        Piece::title(list.thing().to_uppercase(), named(list, key, item)),
        Piece::field("engine", engine.name()),
        Piece::Blank,
    ];

    for (label, field, _) in meant(list) {
        said.extend(field_of(label, field, item.get(*field)));
    }
    said.push(Piece::Blank);

    let warned = warnings(engine, list, item);
    if !warned.is_empty() {
        said.push(Piece::heading("WHAT TO LOOK AT"));
        said.extend(warned.into_iter().map(Piece::warning));
        said.push(Piece::Blank);
    }

    said.push(Piece::heading("WHAT EACH FIELD MEANS"));
    for (label, _, meaning) in meant(list) {
        said.push(Piece::text(format!("{label}: {meaning}")));
    }
    said.push(Piece::Blank);
    said.push(Piece::field("object", key));
    said.push(Piece::key(finding_key(key)));
    said
}

fn named(list: List, key: &str, item: &Value) -> String {
    match (list, field_list(item, "tags").as_slice()) {
        (List::Images, []) => format!("{}, which no tag names", id_of(key)),
        (List::Images, tags) => tags.join(", "),
        _ => id_of(key).to_string(),
    }
}

fn field_of(label: &str, field: &str, value: Option<&Value>) -> Vec<Piece> {
    let many: Vec<String> = match value {
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map_or_else(|| item.to_string(), str::to_string)
            })
            .collect(),
        Some(Value::Object(named)) => named
            .iter()
            .map(|(name, value)| format!("{name}={}", value.as_str().unwrap_or_default()))
            .collect(),
        _ => return vec![Piece::field(label, one(field, value))],
    };

    match many.as_slice() {
        [] => vec![Piece::field(label, "none")],
        [only] => vec![Piece::field(label, only.clone())],
        _ => {
            let mut said = vec![Piece::field(label, format!("{} in all", many.len()))];
            said.extend(
                many.into_iter()
                    .map(|one| Piece::field("", format!("\u{b7} {one}"))),
            );
            said
        }
    }
}

fn one(field: &str, value: Option<&Value>) -> String {
    match (field, value) {
        ("value_redacted", Some(Value::Bool(true))) => "hidden: never read".to_string(),
        (_, Some(Value::Bool(true))) => "yes".to_string(),
        (_, Some(Value::Bool(false))) => "no".to_string(),
        (_, Some(Value::String(text))) if !text.is_empty() => text.clone(),
        (_, Some(Value::Number(number))) => number.to_string(),
        _ => NOT_PRINTED.to_string(),
    }
}
