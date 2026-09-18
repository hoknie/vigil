use serde_json::Value;
use vigil_model::{Change, Finding};
use vigil_rules::{Rule, RuleContext};

use super::engine_finding::{EngineFinding, build, named, note};
use crate::helpers::parts_of;
use crate::types::{Engine, Lifecycle};

pub struct Appeared(pub Lifecycle);

impl Rule for Appeared {
    fn name(&self) -> &'static str {
        self.0.rule
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (engine, subject, _) = parts_of(change.key())?;
        if subject != self.0.subject {
            return None;
        }
        let lifecycle = &self.0;
        let key = change.key();

        let (said, before, after, evidence, title) = match change {
            Change::Added { after, .. } => (
                &lifecycle.new,
                None,
                Some(after),
                Vec::new(),
                format!(
                    "{} has a new {}: {}",
                    engine.name(),
                    lifecycle.thing,
                    named(key, after)
                ),
            ),
            Change::Removed { before, .. } => (
                &lifecycle.removed,
                Some(before),
                None,
                Vec::new(),
                format!(
                    "{} no longer has the {} {}",
                    engine.name(),
                    lifecycle.thing,
                    named(key, before)
                ),
            ),
            Change::Changed { before, after, .. } => {
                let moved = moved(lifecycle.matters, before, after);
                if moved.is_empty() {
                    return None;
                }
                (
                    &lifecycle.changed,
                    Some(before),
                    Some(after),
                    moved.iter().map(|said| note(said.clone())).collect(),
                    changed_title(engine, lifecycle, key, after, &moved),
                )
            }
        };

        Some(build(
            EngineFinding {
                kind: said.0,
                severity: said.1.clone(),
                rule: lifecycle.rule,
                key,
                object: lifecycle.object,
                title,
                before: before.cloned(),
                after: after.cloned(),
                evidence,
            },
            ctx,
        ))
    }
}

fn changed_title(
    engine: Engine,
    lifecycle: &Lifecycle,
    key: &str,
    after: &Value,
    moved: &[String],
) -> String {
    let fields: Vec<&str> = moved
        .iter()
        .filter_map(|said| said.split_once(':').map(|(field, _)| field))
        .collect();
    format!(
        "The {} {} of {} changed its {}",
        lifecycle.thing,
        named(key, after),
        engine.name(),
        fields.join(" and ")
    )
}

fn moved(matters: &[&str], before: &Value, after: &Value) -> Vec<String> {
    matters
        .iter()
        .filter(|field| before.get(**field) != after.get(**field))
        .map(|field| {
            format!(
                "{}: {} \u{2192} {}",
                field.replace('_', " "),
                shown(before.get(*field)),
                shown(after.get(*field))
            )
        })
        .collect()
}

fn shown(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => "nothing".to_string(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) if items.is_empty() => "none".to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map_or_else(|| item.to_string(), str::to_string)
            })
            .collect::<Vec<String>>()
            .join(", "),
        Some(other) => other.to_string(),
    }
}
