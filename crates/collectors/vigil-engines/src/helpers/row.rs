use serde_json::Value;

use crate::types::{Engine, Subject};

pub fn parts_of(key: &str) -> Option<(Engine, Subject, &str)> {
    let mut parts = key.splitn(3, '|');
    let engine = Engine::named(parts.next()?)?;
    let subject = Subject::named(parts.next()?)?;
    let id = parts.next().filter(|id| !id.is_empty())?;

    Some((engine, subject, id))
}

pub fn field_text<'a>(item: &'a Value, field: &str) -> Option<&'a str> {
    item.get(field)
        .and_then(Value::as_str)
        .filter(|said| !said.is_empty())
}

pub fn field_list<'a>(item: &'a Value, field: &str) -> Vec<&'a str> {
    item.get(field)
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn field_flag(item: &Value, field: &str) -> bool {
    item.get(field).and_then(Value::as_bool).unwrap_or(false)
}

pub fn project_of(item: &Value) -> Option<&str> {
    field_text(item, "project")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_key_of_this_reading_names_its_engine_its_subject_and_the_thing_itself() {
        assert_eq!(
            parts_of("docker|registry|registry.local:5000"),
            Some((Engine::Docker, Subject::Registry, "registry.local:5000"))
        );
        assert_eq!(
            parts_of("podman|image|sha256:ab|cd"),
            Some((Engine::Podman, Subject::Image, "sha256:ab|cd")),
            "the engine and the subject are the first two parts, and whatever an id holds is \
             the id"
        );
        for foreign in [
            "container|3ab1",
            "docker|images|x",
            "docker|image|",
            "k8s|pod|x",
        ] {
            assert_eq!(parts_of(foreign), None, "{foreign}");
        }
    }

    #[test]
    fn a_field_the_engine_left_empty_or_null_reads_as_nothing_and_never_as_a_blank_value() {
        let item = json!({"project": null, "driver": "", "subnets": ["10.89.0.0/24"]});

        assert_eq!(project_of(&item), None);
        assert_eq!(field_text(&item, "driver"), None);
        assert_eq!(field_list(&item, "subnets"), vec!["10.89.0.0/24"]);
        assert!(field_list(&item, "nothing").is_empty());
        assert!(!field_flag(&item, "internal"));
    }
}
