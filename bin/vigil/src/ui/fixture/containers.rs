use serde_json::json;
use vigil_model::Snapshot;

const AT: &str = "2026-09-09T09:00:00.000Z";

pub fn containers() -> Snapshot {
    Snapshot::new("containers", AT)
        .with(
            "container|3ab1c0f2d4e5",
            json!({
                "id": "3ab1c0f2d4e5a6b7c8d9e0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5",
                "runtime": "docker",
                "exe": "/usr/sbin/nginx",
                "capabilities_effective": "00000000a80425fb",
                "host_paths": ["/srv/www", "/var/lib/docker/containers/3ab1/resolv.conf"],
                "host_paths_truncated": false,
                "mounts_readable": true,
            }),
        )
        .with(
            "container|9f2e8d7c6b5a",
            json!({
                "id": "9f2e8d7c6b5a4039281706f5e4d3c2b1a0998877665544332211ffeeddccbbaa",
                "runtime": "docker",
                "exe": "/usr/local/bin/agent",
                "capabilities_effective": "000001ffffffffff",
                "host_paths": ["/run/docker.sock"],
                "host_paths_truncated": false,
                "mounts_readable": true,
            }),
        )
        .with(
            "container-socket|/run/docker.sock",
            json!({"path": "/run/docker.sock", "mode": "0660", "uid": 0, "gid": 999}),
        )
}
