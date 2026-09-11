use serde_json::json;
use vigil_model::Snapshot;

pub fn accounts() -> Snapshot {
    Snapshot::new("users", "2026-09-09T09:00:00.000Z")
        .with(
            "account|root",
            json!({
                "name": "root", "uid": 0, "gid": 0, "home": "/root", "shell": "/bin/sh",
                "interactive": true, "password": "disabled", "password_permits_login": false,
                "password_last_change_day": 19000, "password_max_age_days": null,
                "account_expires_day": null, "shadow_readable": true,
            }),
        )
        .with(
            "account|backdoor",
            json!({
                "name": "backdoor", "uid": 0, "gid": 0, "home": "/home/backdoor",
                "shell": "/bin/bash", "interactive": true, "password": "set",
                "password_permits_login": true, "password_last_change_day": 20700,
                "password_max_age_days": null, "account_expires_day": null,
                "shadow_readable": true,
            }),
        )
        .with(
            "account|deploy",
            json!({
                "name": "deploy", "uid": 1000, "gid": 1000, "home": "/home/deploy",
                "shell": "/bin/bash", "interactive": true, "password": "locked",
                "password_permits_login": false, "password_last_change_day": 19100,
                "password_max_age_days": null, "account_expires_day": null,
                "shadow_readable": true,
            }),
        )
        .with(
            "account|www-data",
            json!({
                "name": "www-data", "uid": 33, "gid": 33, "home": "/var/www",
                "shell": "/usr/sbin/nologin", "interactive": false, "password": "locked",
                "password_permits_login": false, "password_last_change_day": null,
                "password_max_age_days": null, "account_expires_day": null,
                "shadow_readable": true,
            }),
        )
        .with(
            "account|contractor",
            json!({
                "name": "contractor", "uid": 1001, "gid": 1001, "home": "/home/contractor",
                "shell": "/bin/bash", "interactive": true, "password": "set",
                "password_permits_login": true, "password_last_change_day": 19500,
                "password_max_age_days": 90, "account_expires_day": 20000,
                "shadow_readable": true,
            }),
        )
        .with(
            "account|svc-runner",
            json!({
                "name": "svc-runner", "uid": 998, "gid": 998, "home": "/var/lib/runner",
                "shell": "/bin/sh", "interactive": true, "password": null,
                "password_permits_login": null, "password_last_change_day": null,
                "password_max_age_days": null, "account_expires_day": null,
                "shadow_readable": false,
            }),
        )
        .with(
            "group|root",
            json!({
                "name": "root", "gid": 0, "members": ["backdoor", "root"],
                "privileged": true, "privilege": "the superuser's own group",
            }),
        )
        .with(
            "group|wheel",
            json!({
                "name": "wheel", "gid": 10, "members": ["deploy"],
                "privileged": true, "privilege": "may run commands as any user",
            }),
        )
        .with(
            "group|docker",
            json!({
                "name": "docker", "gid": 998, "members": ["deploy"], "privileged": true,
                "privilege": "may start a container that mounts the host filesystem — root by another route",
            }),
        )
        .with(
            "group|users",
            json!({
                "name": "users", "gid": 100, "members": [], "privileged": false,
                "privilege": null,
            }),
        )
        .with(
            "sudoer|%wheel",
            json!({
                "who": "%wheel", "group": true, "nopasswd": false, "all_commands": true,
                "spec_redacted": false,
                "rules": [{
                    "source": "/etc/sudoers", "spec": "ALL=(ALL) ALL",
                    "spec_redacted": false, "nopasswd": false, "all_commands": true,
                }],
            }),
        )
        .with(
            "sshkey|deploy|SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ",
            json!({
                "user": "deploy", "uid": 1000, "algorithm": "ssh-ed25519",
                "fingerprint": "SHA256:3VaOaGZ8sBqrDLBz5nfCTd3bAqTL1s1a7uYRoOoJcVQ",
                "comment": "person@laptop", "options": null,
                "source": "/home/deploy/.ssh/authorized_keys", "readable": true,
            }),
        )
        .with(
            "sshkey|contractor|SHA256:kAeGqLq6Y4oQDPLEkLrGuLpZ1hgP4yTtCqHkZbLgAcQ",
            json!({
                "user": "contractor", "uid": 1001, "algorithm": "ssh-ed25519",
                "fingerprint": "SHA256:kAeGqLq6Y4oQDPLEkLrGuLpZ1hgP4yTtCqHkZbLgAcQ",
                "comment": null, "options": "from=\"10.0.0.0/8\",no-pty",
                "source": "/home/contractor/.ssh/authorized_keys", "readable": true,
            }),
        )
        .with(
            "sshkey|backup|unreadable",
            json!({
                "user": "backup", "uid": 1001,
                "source": "/home/backup/.ssh/authorized_keys", "readable": false,
            }),
        )
        .with(
            "session|deploy|pts/0",
            json!({
                "user": "deploy", "uid": 1000, "line": "pts/0", "from": "10.0.0.5",
                "remote": true, "pid": 4242, "session_id": "83", "service": "sshd",
                "type": "tty", "class": "user", "state": "active", "attended": true,
                "seen_by": ["logind", "utmp"],
            }),
        )
        .with(
            "session|root|logind:84",
            json!({
                "user": "root", "uid": 0, "line": "", "from": "", "remote": false,
                "pid": 900, "session_id": "84", "service": "systemd-user",
                "type": "unspecified", "class": "background", "state": "closing",
                "attended": false, "seen_by": ["logind"],
            }),
        )
        .with(
            "session|unknown|pts/3",
            json!({
                "user": "unknown", "uid": null, "line": "pts/3", "from": "203.0.113.9",
                "remote": true, "pid": 5150, "session_id": "85", "service": "sshd",
                "type": "tty", "class": "user", "state": "opening", "attended": true,
                "seen_by": ["utmp"],
            }),
        )
        .with(
            "session-source|logind",
            json!({
                "source": "logind", "path": "/run/systemd/sessions", "present": true,
                "read": true, "answers": true, "sessions": 2, "reason": null,
            }),
        )
        .with(
            "session-source|utmp",
            json!({
                "source": "utmp", "path": "/run/utmp", "present": false, "read": false,
                "answers": false, "sessions": 0,
                "reason": "neither /run/utmp nor /var/run/utmp is on this host: nothing writes a utmp login record here",
            }),
        )
}
