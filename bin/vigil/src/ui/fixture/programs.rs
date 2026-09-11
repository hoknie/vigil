use serde_json::json;
use vigil_model::Snapshot;

pub fn processes() -> Snapshot {
    Snapshot::new("processes", "2026-09-09T09:00:00.000Z")
        .with(
            "exec|/usr/sbin/nginx|root",
            json!({
                "exe": "/usr/sbin/nginx", "exe_deleted": false, "uid": 0, "user": "root",
                "parents": ["/usr/lib/systemd/systemd"], "parents_truncated": false,
                "cmdline": "nginx: master process", "cmdline_varies": false,
                "cmdline_redacted": false, "writable_path": false, "exe_resolved": true,
            }),
        )
        .with(
            "exec|/usr/sbin/nginx|www-data",
            json!({
                "exe": "/usr/sbin/nginx", "exe_deleted": false, "uid": 33, "user": "www-data",
                "parents": ["/usr/sbin/nginx"], "parents_truncated": false,
                "cmdline": null, "cmdline_varies": true, "cmdline_redacted": false,
                "writable_path": false, "exe_resolved": true,
            }),
        )
        .with(
            "exec|/tmp/.x/nc|www-data",
            json!({
                "exe": "/tmp/.x/nc", "exe_deleted": true, "uid": 33, "user": "www-data",
                "parents": ["/bin/sh"], "parents_truncated": false,
                "cmdline": "nc -l -p 4444 -e [redacted]", "cmdline_varies": false,
                "cmdline_redacted": true, "writable_path": true, "exe_resolved": true,
            }),
        )
        .with(
            "exec|/opt/app/server|4242",
            json!({
                "exe": "/opt/app/server", "exe_deleted": false, "uid": 4242, "user": null,
                "parents": [], "parents_truncated": false,
                "cmdline": null, "cmdline_varies": false, "cmdline_redacted": false,
                "writable_path": false, "exe_resolved": true,
            }),
        )
        .with(
            "processes|unresolved",
            json!({
                "exe_resolved": false,
                "reason": "some executables could not be read: those programs are not in this reading (needs CAP_SYS_PTRACE, or root without a restricted capability set)",
            }),
        )
}
