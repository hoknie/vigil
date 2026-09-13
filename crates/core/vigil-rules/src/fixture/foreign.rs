use serde_json::{Value, json};

pub fn of_another_collector(key: &str) -> Value {
    json!({
        "protocol": "tcp",
        "address": "0.0.0.0",
        "port": 443,
        "uid": 0,
        "user": "root",
        "key_it_was_read_under": key,
        "process": { "exe": "/usr/sbin/nginx", "exe_deleted": false },
        "owner_resolved": true,
    })
}
