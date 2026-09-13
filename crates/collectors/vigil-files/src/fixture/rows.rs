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
