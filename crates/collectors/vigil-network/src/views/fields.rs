use serde_json::Value;
use vigil_view::basename;

use crate::types::SocketView;

pub(super) fn protocol<'a>(item: &'a Value, key: &'a str) -> &'a str {
    item.get("protocol")
        .and_then(Value::as_str)
        .unwrap_or_else(|| key.split('|').next().unwrap_or("?"))
}

pub(super) fn endpoint(item: &Value, key: &str) -> String {
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

pub(super) fn user(item: &Value) -> String {
    match item.get("user").and_then(Value::as_str) {
        Some(user) => user.to_string(),
        None => match item.get("uid").and_then(Value::as_u64) {
            Some(uid) => format!("uid {uid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn program(item: &Value) -> String {
    let view = SocketView::new(item);
    if !view.owner_resolved() {
        return "owner not resolved".to_string();
    }

    let executable = view.executable().unwrap_or("unknown");
    match view.executable_deleted() {
        true => format!("{} (deleted)", basename(executable)),
        false => basename(executable).to_string(),
    }
}

pub(super) fn command(item: &Value) -> String {
    let view = SocketView::new(item);
    let Some(line) = view.command_line() else {
        return String::new();
    };
    match view.command_line_redacted() {
        true => format!("{line}  ← part hidden before writing"),
        false => line.to_string(),
    }
}

pub(super) fn holder(item: &Value) -> Option<&str> {
    let view = SocketView::new(item);
    match view.owner_resolved() {
        true => view.executable(),
        false => None,
    }
}
