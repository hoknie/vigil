use serde_json::json;
use vigil_model::Snapshot;

pub fn launches() -> Snapshot {
    Snapshot::new("launches", "2026-09-09T09:00:00.000Z")
        .with(
            "run|alice|/usr/bin/nmap",
            json!({
                "user": "alice", "auid": 1001, "exe": "/usr/bin/nmap",
                "exe_lossy": false, "exe_present": true, "exe_shown": true,
                "writable_path": false, "first_seen": "2026-09-09T08:59:00.000Z",
                "audit_id": "1757419203.412:3421", "arguments": null,
                "arguments_redacted": false,
            }),
        )
        .with(
            "run|root|/dev/shm/payload",
            json!({
                "user": "root", "auid": 0, "exe": "/dev/shm/payload",
                "exe_lossy": false, "exe_present": false, "exe_shown": true,
                "writable_path": true, "first_seen": "2026-09-09T08:58:00.000Z",
                "audit_id": "1757419204.900:3422", "arguments": "payload --quiet",
                "arguments_redacted": false,
            }),
        )
        .with(
            "run|4242|/usr/bin/mysql",
            json!({
                "user": null, "auid": 4242, "exe": "/usr/bin/mysql",
                "exe_lossy": false, "exe_present": null, "exe_shown": false,
                "writable_path": false, "first_seen": "2026-09-09T08:57:00.000Z",
                "audit_id": "1757419205.100:3423", "arguments": "mysql [redacted]",
                "arguments_redacted": true,
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
                "reason": "the audit plugin dropped the oldest events to stay under its spool size; launches from that window were never read",
            }),
        )
        .with(
            "launches|unnamed",
            json!({
                "named": false,
                "reason": "some launches carried no path this build could resolve: those programs are not in this reading",
            }),
        )
        .with(
            "launches|capped",
            json!({
                "named": false,
                "reason": "the limit of 20000 pairs of person and program is reached: launches are no longer being recorded",
            }),
        )
}
