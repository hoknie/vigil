use serde_json::{Value, json};

pub fn unit(name: &str, command: &str, run_as: &str) -> Value {
    json!({
        "name": name,
        "type": name.rsplit_once('.').map(|(_, kind)| kind).unwrap_or("unit"),
        "path": format!("/lib/systemd/system/{name}"),
        "readable": true,
        "description": "A service",
        "commands": [command],
        "commands_redacted": false,
        "run_as": run_as,
    })
}

pub fn timer(name: &str, on_calendar: &[&str], on_boot: Option<&str>) -> Value {
    json!({
        "name": name,
        "type": "timer",
        "path": format!("/lib/systemd/system/{name}"),
        "readable": true,
        "description": "A timer",
        "on_calendar": on_calendar,
        "on_boot": on_boot,
        "activates": name.trim_end_matches(".timer").to_string() + ".service",
    })
}

pub fn cron_job(source: &str, user: &str, schedule: &str, command: &str) -> Value {
    json!({
        "source": source,
        "user": user,
        "schedule": schedule,
        "command": command,
        "command_redacted": false,
    })
}

pub fn kernel_module(name: &str, size: u64) -> Value {
    json!({
        "name": name,
        "size": size,
        "dependencies": [],
        "state": "Live",
    })
}

pub fn script(path: &str, family: &str, digest: &str) -> Value {
    json!({
        "path": path,
        "family": family,
        "present": true,
        "readable": true,
        "sha256": digest,
        "size": 512,
        "mode": "0644",
        "uid": 0,
        "gid": 0,
    })
}

pub fn preload(entries: &[&str]) -> Value {
    json!({
        "path": "/etc/ld.so.preload",
        "present": !entries.is_empty(),
        "readable": true,
        "entries": entries,
        "sha256": match entries.is_empty() {
            true => Value::Null,
            false => json!("0f".repeat(32)),
        },
    })
}
