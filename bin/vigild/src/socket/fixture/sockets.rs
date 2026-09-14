use serde_json::json;
use vigil_model::Snapshot;

pub fn snapshot() -> Snapshot {
    Snapshot::new("ports", "2026-09-09T09:00:00.000Z")
        .with(
            "tcp|0.0.0.0:443",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 443, "uid": 0, "user": "root",
                "process": {
                    "pid": 1042,
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
                    "pid": 30211,
                    "exe": "/tmp/.x/nc", "exe_deleted": true,
                    "cmdline": "nc -l -p 4444", "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "tcp|0.0.0.0:9000",
            json!({
                "protocol": "tcp", "address": "0.0.0.0", "port": 9000, "uid": 33,
                "user": "www-data",
                "process": {
                    "pid": 2291,
                    "exe": null, "exe_deleted": false,
                    "cmdline": null, "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "tcp|127.0.0.1:5432",
            json!({
                "protocol": "tcp", "address": "127.0.0.1", "port": 5432, "uid": 106,
                "user": null, "process": null, "owner_resolved": false,
            }),
        )
        .with(
            "tcp6|:::443",
            json!({
                "protocol": "tcp6", "address": "::", "port": 443, "uid": 0, "user": "root",
                "process": {
                    "pid": 1042,
                    "exe": "/usr/sbin/nginx", "exe_deleted": false,
                    "cmdline": "nginx -g daemon off;", "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "tcp6|:::8080",
            json!({
                "protocol": "tcp6", "address": "::", "port": 8080, "uid": 106,
                "user": null, "process": null, "owner_resolved": false,
            }),
        )
        .with(
            "udp|0.0.0.0:53",
            json!({
                "protocol": "udp", "address": "0.0.0.0", "port": 53, "uid": 101,
                "user": "systemd-resolve",
                "process": {
                    "pid": 640,
                    "exe": "/lib/systemd/systemd-resolved", "exe_deleted": false,
                    "cmdline": "/lib/systemd/systemd-resolved", "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "udp|0.0.0.0:68",
            json!({
                "protocol": "udp", "address": "0.0.0.0", "port": 68, "uid": 106,
                "user": null, "process": null, "owner_resolved": false,
            }),
        )
        .with(
            "udp6|:::546",
            json!({
                "protocol": "udp6", "address": "::", "port": 546, "uid": 0, "user": "root",
                "process": {
                    "pid": 701,
                    "exe": "/usr/sbin/dhclient", "exe_deleted": false,
                    "cmdline": "dhclient -6 --password [redacted]", "cmdline_redacted": true,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "udp6|:::5353",
            json!({
                "protocol": "udp6", "address": "::", "port": 5353, "uid": 106,
                "user": null, "process": null, "owner_resolved": false,
            }),
        )
        .with(
            "unix|/run/docker.sock",
            json!({
                "protocol": "unix", "type": "stream", "path": "/run/docker.sock",
                "abstract": false, "uid": 0, "user": "root",
                "process": {
                    "pid": 980,
                    "exe": "/usr/bin/dockerd", "exe_deleted": false,
                    "cmdline": "dockerd --host unix:///run/docker.sock",
                    "cmdline_redacted": false,
                },
                "owner_resolved": true,
            }),
        )
        .with(
            "unix|@/tmp/.X11-unix/X0",
            json!({
                "protocol": "unix", "type": "stream", "path": "@/tmp/.X11-unix/X0",
                "abstract": true, "uid": null, "user": null, "process": null,
                "owner_resolved": false,
            }),
        )
        .with(
            "unix|unnamed",
            json!({"protocol": "unix", "count": 1, "owner_resolved": false}),
        )
}
