use serde_json::{Value, json};

use vigil_rules::is_writable_path;

pub fn program(executable: &str, user: &str, uid: u64, parents: &[&str]) -> Value {
    json!({
        "exe": executable,
        "exe_deleted": false,
        "uid": uid,
        "user": user,
        "parents": parents,
        "parents_truncated": false,
        "cmdline": format!("{executable} --serve"),
        "cmdline_varies": false,
        "cmdline_redacted": false,
        "writable_path": is_writable_path(executable),
        "exe_resolved": true,
    })
}

pub fn program_with_deleted_binary(executable: &str, user: &str, uid: u64) -> Value {
    let mut value = program(executable, user, uid, &[]);
    value["exe_deleted"] = json!(true);
    value
}
