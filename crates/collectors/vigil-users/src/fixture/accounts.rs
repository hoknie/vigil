use serde_json::{Value, json};

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
