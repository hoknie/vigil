use serde_json::json;
use vigil_model::Snapshot;

pub fn persistence() -> Snapshot {
    Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
        .with(
            "unit|nginx.service",
            json!({
                "name": "nginx.service", "type": "service",
                "path": "/lib/systemd/system/nginx.service", "readable": true,
                "description": "A high performance web server",
                "commands": ["/usr/sbin/nginx -g 'daemon off;'"],
                "commands_redacted": false, "run_as": "root",
            }),
        )
        .with(
            "timer|logrotate.timer",
            json!({
                "name": "logrotate.timer", "type": "timer",
                "path": "/lib/systemd/system/logrotate.timer", "readable": true,
                "description": "Daily rotation of log files", "on_calendar": "daily",
                "on_boot": false, "activates": "logrotate.service",
            }),
        )
        .with(
            "cron|/etc/crontab|root|/usr/local/bin/backup --to /srv",
            json!({
                "source": "/etc/crontab", "user": "root", "schedule": "@daily",
                "command": "/usr/local/bin/backup --to /srv", "command_redacted": false,
            }),
        )
        .with(
            "cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant",
            json!({
                "source": "/var/spool/cron/crontabs/www-data", "user": "www-data",
                "schedule": "*/5 * * * *", "command": "/tmp/.x/implant",
                "command_redacted": false,
            }),
        )
        .with(
            "module|overlay",
            json!({
                "name": "overlay", "size": 155648, "dependencies": [], "state": "Live",
            }),
        )
        .with(
            "script|/etc/profile",
            json!({
                "path": "/etc/profile", "family": "profile", "present": true, "readable": true,
                "sha256": "9f2c1b3d4e5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c",
                "size": 581, "mode": "0644", "uid": 0, "gid": 0,
            }),
        )
        .with(
            "preload|/etc/ld.so.preload",
            json!({
                "path": "/etc/ld.so.preload", "present": true, "readable": true,
                "entries": ["/usr/local/lib/libjackit.so"],
                "sha256": "1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b",
            }),
        )
}
