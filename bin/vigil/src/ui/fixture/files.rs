use serde_json::json;
use vigil_model::Snapshot;

const AT: &str = "2026-09-09T09:00:00.000Z";

pub fn files() -> Snapshot {
    Snapshot::new("files", AT)
        .with(
            "file|/etc/ssh/sshd_config",
            json!({
                "path": "/etc/ssh/sshd_config", "present": true, "readable": true,
                "sha256": "87e95e7564445110057c3e0563e80077ccdab840ac81fccc5e66aaef991bb298",
                "size": 53, "mode": "0600", "uid": 0, "gid": 0, "over_the_ceiling": false,
            }),
        )
        .with(
            "file|/etc/hosts",
            json!({
                "path": "/etc/hosts", "present": true, "readable": true,
                "sha256": "00e987d761e447af9e2d73ff09ccc896273948c756a71d5bfb8db3a73dc2eb54",
                "size": 48, "mode": "0644", "uid": 0, "gid": 0, "over_the_ceiling": false,
            }),
        )
        .with(
            "file|/usr/bin/newgrp",
            json!({
                "path": "/usr/bin/newgrp", "present": true, "readable": true,
                "sha256": "5f0c5b3f0a58f0a9ef4b8e4e7e2a4fd3f1b0a9c8d7e6f5a4b3c2d1e0f9a8b7c6",
                "size": 39_192, "mode": "4755", "uid": 0, "gid": 0, "over_the_ceiling": false,
            }),
        )
        .with(
            "file|/etc/pam.d/sshd",
            json!({
                "path": "/etc/pam.d/sshd", "present": false, "readable": false,
                "sha256": null, "size": 0, "mode": null, "uid": null, "gid": null,
                "over_the_ceiling": false,
            }),
        )
        .with(
            "directory|/usr/bin",
            json!({"path": "/usr/bin", "present": true, "mode": "0755", "uid": 0, "gid": 0}),
        )
        .with(
            "directory|/usr/local/bin",
            json!({"path": "/usr/local/bin", "present": true, "mode": "0775", "uid": 0, "gid": 0}),
        )
}
