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
                "cmdline": "nc -l -p 4444", "cmdline_varies": false, "cmdline_redacted": false,
                "writable_path": true, "exe_resolved": true,
            }),
        )
        .with(
            "processes|unresolved",
            json!({
                "exe_resolved": false,
                "reason": "3 process(es) could not be read: needs CAP_SYS_PTRACE",
            }),
        )
}

pub fn launches() -> Snapshot {
    Snapshot::new("launches", "2026-09-09T09:00:00.000Z")
        .with(
            "run|alice|/usr/bin/nmap",
            json!({
                "user": "alice", "auid": 1001, "exe": "/usr/bin/nmap", "exe_lossy": false,
                "exe_present": true, "writable_path": false,
                "first_seen": "2026-09-09T08:59:00.000Z", "audit_id": 4242,
                "arguments": null, "arguments_redacted": false,
            }),
        )
        .with(
            "run|root|/dev/shm/payload",
            json!({
                "user": "root", "auid": 0, "exe": "/dev/shm/payload", "exe_lossy": false,
                "exe_present": false, "writable_path": true,
                "first_seen": "2026-09-09T08:58:00.000Z", "audit_id": 4243,
                "arguments": "payload --quiet", "arguments_redacted": false,
            }),
        )
        .with(
            "launches|source",
            json!({
                "named": false, "from": "audit plugin",
                "reason": "launches arrive through the plugin auditd starts",
            }),
        )
        .with(
            "launches|dropping",
            json!({
                "named": false,
                "reason": "the audit plugin dropped the oldest events to stay under its spool size",
            }),
        )
}
