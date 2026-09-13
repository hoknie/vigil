use serde_json::{Value, json};

pub fn socket(address: &str, port: u64, executable: &str, user: &str) -> Value {
    json!({
        "protocol": "tcp",
        "address": address,
        "port": port,
        "uid": 33,
        "user": user,
        "process": {
            "exe": executable,
            "exe_deleted": false,
            "cmdline": format!("{executable} -g daemon off;"),
            "cmdline_redacted": false,
        },
        "owner_resolved": true,
    })
}

pub fn socket_with_deleted_binary(address: &str, port: u64, executable: &str) -> Value {
    let mut value = socket(address, port, executable, "www-data");
    value["process"]["exe_deleted"] = json!(true);
    value
}

pub fn unix_socket(path: &str, executable: &str, user: &str) -> Value {
    json!({
        "protocol": "unix",
        "type": "stream",
        "path": path,
        "abstract": path.starts_with('@'),
        "uid": 0,
        "user": user,
        "process": {
            "exe": executable,
            "exe_deleted": false,
            "cmdline": executable,
            "cmdline_redacted": false,
        },
        "owner_resolved": true,
    })
}

pub fn socket_without_owner(address: &str, port: u64) -> Value {
    json!({
        "protocol": "tcp",
        "address": address,
        "port": port,
        "uid": 0,
        "user": "root",
        "process": Value::Null,
        "owner_resolved": false,
    })
}
