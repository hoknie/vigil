use serde_json::{Value, json};

pub fn container(executable: &str, capabilities: &str, host_paths: &[&str]) -> Value {
    json!({
        "id": "3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5",
        "runtime": "docker",
        "exe": executable,
        "capabilities_effective": capabilities,
        "host_paths": host_paths,
        "host_paths_truncated": false,
        "mounts_readable": true,
    })
}

pub fn runtime_socket(path: &str, mode: &str) -> Value {
    json!({
        "path": path,
        "mode": mode,
        "uid": 0,
        "gid": 999,
    })
}
