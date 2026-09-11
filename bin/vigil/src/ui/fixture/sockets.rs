use serde_json::json;
use vigil_model::Snapshot;

pub fn snapshot() -> Snapshot {
    Snapshot::new("ports", "2026-09-09T09:00:00.000Z")
        .with(
            "tcp|0.0.0.0:443",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 443, "uid": 0, "user": "root",
                "process": {
                    "exe": "/usr/sbin/nginx", "exe_deleted": false,
                    "cmdline": "nginx -g daemon off;", "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "tcp|0.0.0.0:4444",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 4444, "uid": 33,
                "user": "www-data",
                "process": {
                    "exe": "/tmp/.x/nc", "exe_deleted": true,
                    "cmdline": "nc -l -p 4444", "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
}
