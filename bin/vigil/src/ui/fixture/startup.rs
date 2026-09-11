use serde_json::json;
use vigil_model::Snapshot;

pub fn persistence() -> Snapshot {
    Snapshot::new("persistence", "2026-09-09T09:00:00.000Z")
        .with(
            "unit|multi-user.target",
            json!({
                "name": "multi-user.target", "type": "target",
                "path": "/lib/systemd/system/multi-user.target", "readable": true,
                "description": "Multi-User System",
                "commands": [], "commands_redacted": false, "run_as": "root",
            }),
        )
        .with(
            "unit|sockets.target",
            json!({
                "name": "sockets.target", "type": "target",
                "path": "/lib/systemd/system/sockets.target", "readable": true,
                "description": "Socket Units",
                "commands": [], "commands_redacted": false, "run_as": "root",
            }),
        )
        .with(
            "unit|dbus.socket",
            json!({
                "name": "dbus.socket", "type": "socket",
                "path": "/lib/systemd/system/dbus.socket", "readable": true,
                "description": "D-Bus System Message Bus Socket",
                "commands": [], "commands_redacted": false, "run_as": "root",
                "wanted_by": ["sockets.target"],
            }),
        )
        .with(
            "unit|nginx.service",
            json!({
                "name": "nginx.service", "type": "service",
                "path": "/lib/systemd/system/nginx.service", "readable": true,
                "description": "A high performance web server",
                "commands": [
                    "/usr/sbin/nginx -t",
                    "/usr/sbin/nginx -g 'daemon off;' --password [redacted]",
                ],
                "commands_redacted": true, "run_as": "www-data",
                "wanted_by": ["multi-user.target", "sockets.target"],
                "wants": ["nss-lookup.target"],
                "required_by": ["web.target"], "requires": ["network.target"],
                "part_of": ["web.target"],
            }),
        )
        .with(
            "unit|rescue-shell.service",
            json!({
                "name": "rescue-shell.service", "type": "service",
                "path": "/etc/systemd/system/rescue-shell.service", "readable": true,
                "description": null,
                "commands": ["/bin/sh"], "commands_redacted": false, "run_as": "root",
            }),
        )
        .with(
            "unit|locked.service",
            json!({
                "name": "locked.service", "type": "service",
                "path": "/etc/systemd/system/locked.service", "readable": false,
                "description": null,
                "commands": [], "commands_redacted": false, "run_as": "root",
            }),
        )
        .with(
            "timer|logrotate.timer",
            json!({
                "name": "logrotate.timer", "type": "timer",
                "path": "/lib/systemd/system/logrotate.timer", "readable": true,
                "description": "Daily rotation of log files",
                "on_calendar": ["daily", "Mon *-*-* 03:15:00"],
                "on_boot": null, "activates": "logrotate.service",
            }),
        )
        .with(
            "timer|watchdog.timer",
            json!({
                "name": "watchdog.timer", "type": "timer",
                "path": "/etc/systemd/system/watchdog.timer", "readable": true,
                "description": null, "on_calendar": [], "on_boot": "15min",
                "activates": "watchdog.service",
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
            "module|nf_nat",
            json!({
                "name": "nf_nat", "size": 49152,
                "dependencies": ["xt_MASQUERADE", "nft_chain_nat"], "state": "-",
            }),
        )
        .with(
            "modules|unreadable",
            json!({
                "readable": false,
                "reason": "/proc/modules could not be read: loaded kernel modules are not being watched",
            }),
        )
        .with(
            "script|/etc/profile",
            json!({
                "path": "/etc/profile", "family": "profile", "present": true, "readable": true,
                "sha256": "9f2c1b3d4e5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c",
                "size": 581, "mode": "0644", "uid": 0, "gid": 0, "shown": true,
            }),
        )
        .with(
            "script|/etc/rc.local",
            json!({
                "path": "/etc/rc.local", "family": "boot", "present": false, "readable": true,
                "sha256": null, "size": 0, "mode": "", "uid": 0, "gid": 0, "shown": true,
            }),
        )
        .with(
            "script|/tmp/.hidden/.bashrc",
            json!({
                "path": "/tmp/.hidden/.bashrc", "family": "profile", "present": null,
                "readable": null, "sha256": null, "size": 0, "mode": "", "uid": 0, "gid": 0,
                "shown": false,
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
