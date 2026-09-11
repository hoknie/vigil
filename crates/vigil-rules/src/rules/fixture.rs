use serde_json::{Value, json};

pub fn socket(address: &str, port: u64, executable: &str, user: &str) -> Value {
    json!({
        "protocol": "tcp",
        "address": address,
        "port": port,
        "uid": 33,
        "user": user,
        "process": {
            "exe": executable,
            "exe_deleted": false,
            "cmdline": format!("{executable} -g daemon off;"),
            "cmdline_redacted": false,
        },
        "owner_resolved": true,
    })
}

pub fn socket_with_deleted_binary(address: &str, port: u64, executable: &str) -> Value {
    let mut value = socket(address, port, executable, "www-data");
    value["process"]["exe_deleted"] = json!(true);
    value
}

pub fn unix_socket(path: &str, executable: &str, user: &str) -> Value {
    json!({
        "protocol": "unix",
        "type": "stream",
        "path": path,
        "abstract": path.starts_with('@'),
        "uid": 0,
        "user": user,
        "process": {
            "exe": executable,
            "exe_deleted": false,
            "cmdline": executable,
            "cmdline_redacted": false,
        },
        "owner_resolved": true,
    })
}

pub fn socket_without_owner(address: &str, port: u64) -> Value {
    json!({
        "protocol": "tcp",
        "address": address,
        "port": port,
        "uid": 0,
        "user": "root",
        "process": Value::Null,
        "owner_resolved": false,
    })
}

pub fn account(name: &str, uid: u64, shell: &str) -> Value {
    let mut value = account_with_password(name, uid, "set", 19000);
    value["shell"] = json!(shell);
    value["interactive"] = json!(!shell.ends_with("nologin") && !shell.ends_with("false"));
    value
}

pub fn account_with_password(name: &str, uid: u64, password: &str, last_change: i64) -> Value {
    json!({
        "name": name,
        "uid": uid,
        "gid": uid,
        "home": format!("/home/{name}"),
        "shell": "/bin/bash",
        "interactive": true,
        "password": password,
        "password_permits_login": password == "set" || password == "empty",
        "password_last_change_day": last_change,
        "password_max_age_days": 99999,
        "account_expires_day": Value::Null,
        "shadow_readable": true,
    })
}

pub fn account_without_shadow(name: &str, uid: u64) -> Value {
    let mut value = account_with_password(name, uid, "set", 19000);
    value["password"] = Value::Null;
    value["password_permits_login"] = Value::Null;
    value["password_last_change_day"] = Value::Null;
    value["password_max_age_days"] = Value::Null;
    value["shadow_readable"] = json!(false);
    value
}

pub fn group(name: &str, members: &[&str], privilege: Option<&str>) -> Value {
    json!({
        "name": name,
        "gid": 27,
        "members": members,
        "privileged": privilege.is_some(),
        "privilege": privilege,
    })
}

pub fn sudoer(who: &str, spec: &str, nopasswd: bool, all_commands: bool) -> Value {
    json!({
        "who": who,
        "group": who.starts_with('%'),
        "rules": [{
            "source": "/etc/sudoers.d/local",
            "spec": spec,
            "spec_redacted": false,
            "nopasswd": nopasswd,
            "all_commands": all_commands,
        }],
        "nopasswd": nopasswd,
        "all_commands": all_commands,
        "spec_redacted": false,
    })
}

pub fn ssh_key(
    user: &str,
    uid: u64,
    fingerprint: &str,
    comment: Option<&str>,
    options: Option<&str>,
) -> Value {
    json!({
        "user": user,
        "uid": uid,
        "algorithm": "ssh-ed25519",
        "fingerprint": fingerprint,
        "comment": comment,
        "options": options,
        "source": format!("/home/{user}/.ssh/authorized_keys"),
        "readable": true,
    })
}

pub fn ssh_keys_unreadable(user: &str, uid: u64) -> Value {
    json!({
        "user": user,
        "uid": uid,
        "source": format!("/home/{user}/.ssh/authorized_keys"),
        "readable": false,
    })
}

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

fn is_writable_path(executable: &str) -> bool {
    ["/tmp/", "/var/tmp/", "/dev/shm/", "/home/", "/run/user/"]
        .iter()
        .any(|writable| executable.starts_with(writable))
}

pub fn program(executable: &str, user: &str, uid: u64, parents: &[&str]) -> Value {
    json!({
        "exe": executable,
        "exe_deleted": false,
        "uid": uid,
        "user": user,
        "parents": parents,
        "parents_truncated": false,
        "cmdline": format!("{executable} --serve"),
        "cmdline_varies": false,
        "cmdline_redacted": false,
        "writable_path": is_writable_path(executable),
        "exe_resolved": true,
    })
}

pub fn program_with_deleted_binary(executable: &str, user: &str, uid: u64) -> Value {
    let mut value = program(executable, user, uid, &[]);
    value["exe_deleted"] = json!(true);
    value
}

pub fn launch(user: &str, auid: u64, executable: &str) -> Value {
    json!({
        "user": user,
        "auid": auid,
        "exe": executable,
        "exe_lossy": false,
        "exe_present": true,
        "writable_path": is_writable_path(executable),
        "first_seen": "2026-09-09T12:00:00.000Z",
        "audit_id": "1757419203.412:3421",
        "arguments": Value::Null,
        "arguments_redacted": false,
    })
}

pub fn launch_of_a_missing_program(user: &str, auid: u64, executable: &str) -> Value {
    let mut value = launch(user, auid, executable);
    value["exe_present"] = json!(false);
    value
}

pub fn firewall_ruleset(tables: u64, base_chains: u64, rules: u64) -> Value {
    json!({
        "version": "1.0.6",
        "families": match tables {
            0 => Vec::new(),
            _ => vec!["inet", "ip"],
        },
        "tables": tables,
        "chains": base_chains + tables,
        "base_chains": base_chains,
        "rules": rules,
        "hooked_on_input": match base_chains {
            0 => 0,
            _ => 1,
        },
        "legacy_backend": false,
    })
}

pub fn firewall_table(family: &str, name: &str, chains: u64, rules: u64) -> Value {
    json!({
        "family": family,
        "name": name,
        "chains": chains,
        "rules": rules,
    })
}

pub fn firewall_chain(family: &str, table: &str, name: &str, policy: &str) -> Value {
    json!({
        "family": family,
        "table": table,
        "name": name,
        "type": "filter",
        "hook": name,
        "priority": 0,
        "policy": policy,
        "rules": 3,
    })
}

pub fn firewall_legacy_backend(tables: &[&str]) -> Value {
    json!({
        "tables": tables,
        "readable": false,
    })
}
