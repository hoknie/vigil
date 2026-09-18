use serde_json::{Value, json};

pub fn watched_file(path: &str, mode: &str, digest: &str) -> Value {
    json!({
        "path": path,
        "present": true,
        "readable": true,
        "sha256": digest.repeat(32),
        "size": 3_281,
        "mode": mode,
        "uid": 0,
        "gid": 0,
        "over_the_ceiling": false,
    })
}

pub fn watched_file_absent(path: &str) -> Value {
    json!({
        "path": path,
        "present": false,
        "readable": false,
        "sha256": Value::Null,
        "size": 0,
        "mode": Value::Null,
        "uid": Value::Null,
        "gid": Value::Null,
        "over_the_ceiling": false,
    })
}

pub fn watched_directory(path: &str, mode: &str) -> Value {
    json!({
        "path": path,
        "present": true,
        "mode": mode,
        "uid": 0,
        "gid": 0,
    })
}

pub fn walked_file(path: &str, by: &str, mode: &str) -> Value {
    let mut row = watched_file(path, mode, "c3");
    row["type"] = json!("file");
    row["found_by"] = json!(by);
    row["complete"] = json!(true);
    row
}

pub fn walk(entry: &str, matched: usize, complete: bool) -> Value {
    json!({
        "entry": entry,
        "kind": "tree",
        "matched": matched,
        "complete": complete,
        "not_entered": [],
        "max_file_size": 1_048_576,
    })
}
