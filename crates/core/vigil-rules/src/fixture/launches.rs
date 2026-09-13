use serde_json::{Value, json};

use crate::helpers::is_writable_path;

pub fn launch(user: &str, auid: u64, executable: &str) -> Value {
    json!({
        "user": user,
        "auid": auid,
        "exe": executable,
        "exe_lossy": false,
        "exe_present": true,
        "writable_path": is_writable_path(executable),
        "first_seen": "2026-09-09T12:00:00.000Z",
        "audit_id": "1757419203.412:3421",
        "arguments": Value::Null,
        "arguments_redacted": false,
    })
}

pub fn launch_of_a_missing_program(user: &str, auid: u64, executable: &str) -> Value {
    let mut value = launch(user, auid, executable);
    value["exe_present"] = json!(false);
    value
}
