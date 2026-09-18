use serde_json::{Value, json};
use vigil_model::{Evidence, Finding, Kind, KnownKind, Severity, State, Subject as Described};
use vigil_rules::RuleContext;

use crate::helpers::{HIDDEN, field_flag, finding_key, parts_of, project_of};

pub struct EngineFinding<'a> {
    pub kind: KnownKind,
    pub severity: Severity,
    pub rule: &'static str,
    pub key: &'a str,
    pub object: &'static str,
    pub title: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub evidence: Vec<Evidence>,
}

pub fn build(spec: EngineFinding<'_>, ctx: &mut RuleContext<'_>) -> Finding {
    let mut redacted = Vec::new();
    for (side, item) in [("/before", &spec.before), ("/after", &spec.after)] {
        if let Some(item) = item {
            redacted.extend(hidden(side, item));
        }
    }

    let mut evidence = spec.evidence;
    if let Some(project) = spec
        .after
        .as_ref()
        .or(spec.before.as_ref())
        .and_then(project_of)
    {
        evidence.push(note(format!("of the compose project {project}")));
    }

    let now = ctx.now.clone();
    Finding {
        event_id: (ctx.mint_event_id)(),
        finding_key: finding_key(spec.key),
        kind: Kind::Known(spec.kind),
        severity: spec.severity,
        state: State::Open,
        observed_at: now.clone(),
        first_seen_at: now,
        occurrences: 1,
        title: spec.title,
        subject: Described {
            object: spec.object.into(),
            key: described(spec.key),
        },
        before: spec.before,
        after: spec.after,
        evidence,
        redacted,
        rule: Some(spec.rule.to_string()),
        labels: Default::default(),
    }
}

pub fn note(value: impl Into<String>) -> Evidence {
    Evidence {
        kind: "note".into(),
        value: value.into(),
    }
}

pub fn named(key: &str, item: &Value) -> String {
    let tags: Vec<&str> = item
        .get("tags")
        .and_then(Value::as_array)
        .map(|tags| tags.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    match (tags.is_empty(), parts_of(key)) {
        (false, _) => tags.join(", "),
        (true, Some((_, _, id))) => id.to_string(),
        (true, None) => key.to_string(),
    }
}

fn described(key: &str) -> Value {
    match parts_of(key) {
        Some((engine, subject, id)) => json!({
            "engine": engine.name(),
            subject.as_str(): id,
        }),
        None => json!({ "key": key }),
    }
}

fn hidden(side: &str, item: &Value) -> Vec<String> {
    let mut said = Vec::new();
    if field_flag(item, "value_redacted") {
        said.push(format!("{side}/value"));
    }
    if !field_flag(item, "labels_redacted") {
        return said;
    }
    if let Some(labels) = item.get("labels").and_then(Value::as_object) {
        for (name, value) in labels {
            if value.as_str() == Some(HIDDEN) {
                said.push(format!("{side}/labels/{}", pointed(name)));
            }
        }
    }
    said
}

fn pointed(name: &str) -> String {
    name.replace('~', "~0").replace('/', "~1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_hidden_on_this_host_is_named_by_a_pointer_to_it_and_its_value_is_not() {
        let item = json!({
            "labels": {"com.shop.api-token": HIDDEN, "role": "web", "a/b~c": HIDDEN},
            "labels_redacted": true,
        });

        assert_eq!(
            hidden("/after", &item),
            vec!["/after/labels/a~1b~0c", "/after/labels/com.shop.api-token"],
            "a reader of the finding sees which label was hidden, and a pointer with a slash \
             in it unescaped would point at a field that is not there"
        );
    }

    #[test]
    fn the_value_of_a_secret_is_listed_as_hidden_because_it_was_never_read() {
        let item = json!({"name": "shop-database-password", "value_redacted": true});

        assert_eq!(hidden("/before", &item), vec!["/before/value"]);
    }

    #[test]
    fn a_label_that_merely_says_redacted_on_a_row_that_hid_nothing_is_not_listed() {
        let item = json!({"labels": {"note": HIDDEN}, "labels_redacted": false});

        assert!(hidden("/after", &item).is_empty());
    }

    #[test]
    fn a_finding_names_its_object_by_the_engine_and_the_thing() {
        assert_eq!(
            described("podman|volume|etc_backup"),
            json!({"engine": "podman", "volume": "etc_backup"})
        );
        assert_eq!(
            named("docker|image|sha256:5f0c1ad8b29e", &json!({"tags": []})),
            "sha256:5f0c1ad8b29e"
        );
        assert_eq!(
            named(
                "docker|image|sha256:18ad",
                &json!({"tags": ["nginx:1.27-alpine"]})
            ),
            "nginx:1.27-alpine"
        );
    }
}
