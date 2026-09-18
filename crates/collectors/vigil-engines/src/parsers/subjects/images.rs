use std::collections::BTreeMap;

use serde_json::{Value, json};

use crate::helpers::{number, said, text, words};
use crate::types::Subject;

const NONE: &str = "<none>";

const ID: &[&str] = &["ID", "Id", "id"];

const DIGEST: &[&str] = &["Digest", "digest"];

const SIZE: &[&str] = &["Size", "size"];

pub fn images(rows: &[Value]) -> BTreeMap<String, Value> {
    let mut read: BTreeMap<String, Value> = BTreeMap::new();

    for row in rows {
        let Some(id) = text(row, ID).map(identified) else {
            continue;
        };
        let tags = tags_of(row);
        let digest = digest_of(row);

        let held = read.entry(id.clone()).or_insert_with(|| {
            json!({
                "subject": Subject::Image.as_str(),
                "id": id,
                "tags": Vec::<String>::new(),
                "untagged": true,
                "digest": Value::Null,
                "size": size_of(row),
            })
        });

        add_tags(held, tags);
        if held["digest"].is_null() {
            held["digest"] = digest;
        }
    }

    read
}

fn add_tags(held: &mut Value, tags: Vec<String>) {
    let Some(Value::Array(already)) = held.get_mut("tags") else {
        return;
    };
    for tag in tags {
        let said = Value::String(tag);
        if !already.contains(&said) {
            already.push(said);
        }
    }
    already.sort_by(|one, other| one.as_str().cmp(&other.as_str()));
    held["untagged"] = json!(already.is_empty());
}

fn identified(id: String) -> String {
    match id.contains(':') || !id.chars().all(|letter| letter.is_ascii_hexdigit()) {
        true => id,
        false => format!("sha256:{id}"),
    }
}

fn tags_of(row: &Value) -> Vec<String> {
    let listed = words(row, &["RepoTags", "Names", "names"]);
    if !listed.is_empty() {
        return listed
            .into_iter()
            .filter(|tag| !tag.contains(NONE))
            .collect();
    }

    let (Some(repository), Some(tag)) = (
        text(row, &["Repository", "repository"]),
        text(row, &["Tag", "tag"]),
    ) else {
        return Vec::new();
    };
    match repository.contains(NONE) || tag.contains(NONE) {
        true => Vec::new(),
        false => vec![format!("{repository}:{tag}")],
    }
}

fn digest_of(row: &Value) -> Value {
    if let Some(digest) = text(row, DIGEST).filter(|said| !said.contains(NONE)) {
        return Value::String(digest);
    }
    match words(row, &["RepoDigests", "repoDigests"])
        .into_iter()
        .filter_map(|said| said.split_once('@').map(|(_, digest)| digest.to_string()))
        .next()
    {
        Some(digest) => Value::String(digest),
        None => Value::Null,
    }
}

fn size_of(row: &Value) -> Value {
    match number(row, SIZE) {
        Some(bytes) => Value::String(bytes.to_string()),
        None => said(text(row, SIZE)),
    }
}
