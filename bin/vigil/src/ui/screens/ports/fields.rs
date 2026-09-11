use serde_json::Value;

pub fn protocol<'a>(item: &'a Value, key: &'a str) -> &'a str {
    item.get("protocol")
        .and_then(Value::as_str)
        .unwrap_or_else(|| key.split('|').next().unwrap_or("?"))
}

pub fn endpoint(item: &Value, key: &str) -> String {
    if let Some(path) = item.get("path").and_then(Value::as_str) {
        return match item.get("abstract").and_then(Value::as_bool) == Some(true) {
            true => format!("{path} (abstract)"),
            false => path.to_string(),
        };
    }
    if let Some(count) = item.get("count").and_then(Value::as_u64) {
        return format!("{count} with no name");
    }
    match (
        item.get("address").and_then(Value::as_str),
        item.get("port").and_then(Value::as_u64),
    ) {
        (Some(address), Some(port)) => format!("{address}:{port}"),
        _ => key
            .split_once('|')
            .map(|(_, rest)| rest)
            .unwrap_or(key)
            .to_string(),
    }
}

pub fn user(item: &Value) -> String {
    match item.get("user").and_then(Value::as_str) {
        Some(user) => user.to_string(),
        None => match item.get("uid").and_then(Value::as_u64) {
            Some(uid) => format!("uid {uid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn program(item: &Value) -> String {
    if item.get("owner_resolved").and_then(Value::as_bool) == Some(false) {
        return "owner not resolved".to_string();
    }

    let process = item.get("process");
    let executable = process
        .and_then(|process| process.get("exe"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let deleted = process
        .and_then(|process| process.get("exe_deleted"))
        .and_then(Value::as_bool)
        == Some(true);

    match deleted {
        true => format!("{} (deleted)", basename(executable)),
        false => basename(executable).to_string(),
    }
}

pub(super) fn command(item: &Value) -> String {
    let Some(process) = item.get("process") else {
        return String::new();
    };
    let line = process
        .get("cmdline")
        .and_then(Value::as_str)
        .unwrap_or_default();
    match process.get("cmdline_redacted").and_then(Value::as_bool) == Some(true) {
        true => format!("{line}  ← part hidden before writing"),
        false => line.to_string(),
    }
}

pub fn basename(path: &str) -> &str {
    match path.rsplit_once('/') {
        Some((_, name)) if !name.is_empty() => name,
        _ => path,
    }
}
