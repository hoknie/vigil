use serde_json::Value;
use vigil_model::Snapshot;

use crate::helpers::{
    field_flag, field_list, field_text, held_of_this_host, parts_of, untagged_image,
};
use crate::parsers::key;
use crate::types::{Engine, List, Subject};

pub(super) fn of_the_list(
    reading: &Snapshot,
    engine: Engine,
    list: List,
) -> impl Iterator<Item = (&String, &Value)> {
    let prefix = format!("{}|{}|", engine.name(), list.subject().as_str());
    reading
        .items
        .range(prefix.clone()..)
        .take_while(move |(key, _)| key.starts_with(&prefix))
}

pub(super) fn engine_row(reading: &Snapshot, engine: Engine) -> Option<&Value> {
    reading
        .items
        .get(&key(engine, Subject::Engine, engine.name()))
}

pub(super) fn said_by_the_engine(reading: &Snapshot, engine: Engine) -> Vec<String> {
    let Some(row) = engine_row(reading, engine) else {
        return Vec::new();
    };
    if !field_flag(row, "read") {
        return Vec::new();
    }

    let mut said = vec![format!(
        "{} {}",
        engine.name(),
        field_text(row, "version").unwrap_or("of a version it did not print")
    )];
    if let Some(storage) = field_text(row, "storage_driver") {
        said.push(storage.to_string());
    }
    said.push(
        match row.get("rootless").and_then(Value::as_bool) {
            Some(true) => "rootless",
            Some(false) => "rootful",
            None => "rootful",
        }
        .to_string(),
    );
    said
}

pub(super) fn id_of(key: &str) -> &str {
    parts_of(key).map_or(key, |(_, _, id)| id)
}

pub(super) fn short(id: &str) -> String {
    let bare = id.strip_prefix("sha256:").unwrap_or(id);
    let cut: String = bare.chars().take(12).collect();
    match id.starts_with("sha256:") {
        true => format!("sha256:{cut}"),
        false => cut,
    }
}

pub(super) fn runs(image_key: &str, image: &Value, container: &Value) -> bool {
    let ran = field_text(container, "image").unwrap_or_default();
    let ran_id = field_text(container, "image_id").unwrap_or_default();
    let id = id_of(image_key);
    let bare = id.strip_prefix("sha256:").unwrap_or(id);

    field_list(image, "tags").contains(&ran)
        || (!ran.is_empty() && bare.starts_with(ran.trim_start_matches("sha256:")))
        || (!ran_id.is_empty() && bare.starts_with(ran_id.trim_start_matches("sha256:")))
}

pub(super) fn in_use(reading: &Snapshot, engine: Engine, image_key: &str, image: &Value) -> usize {
    of_the_list(reading, engine, List::Containers)
        .filter(|(_, container)| runs(image_key, image, container))
        .count()
}

pub(super) fn warnings(engine: Engine, list: List, item: &Value) -> Vec<String> {
    let mut said = Vec::new();
    let held = held_of_this_host(engine, list.subject(), item);
    match list {
        List::Containers => {
            if field_flag(item, "host_network") {
                said.push("It shares the network of this host.".to_string());
            }
            if !held.is_empty() {
                said.push(format!("It mounts {} of this host.", held.join(", ")));
            }
            if let Some(image) = untagged_image(item) {
                said.push(format!("It runs an image no tag names: {image}."));
            }
        }
        List::Volumes if !held.is_empty() => {
            said.push(format!("It binds {} of this host.", held.join(", ")));
        }
        List::Registries if field_flag(item, "insecure") => {
            said.push("The engine may pull from it without TLS.".to_string());
        }
        _ => {}
    }
    said
}
