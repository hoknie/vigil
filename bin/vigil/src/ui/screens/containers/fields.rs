use serde_json::Value;

use super::kind::Kind;
use super::row::Row;

const UNSAID: &str = "—";

pub const SYS_ADMIN: u64 = 1 << 21;

pub fn what(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Container => short(text(row.item, "id")).to_string(),
        Kind::Socket => text(row.item, "path").to_string(),
    }
}

pub fn runtime(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Container => text(row.item, "runtime").to_string(),
        Kind::Socket => UNSAID.to_string(),
    }
}

pub fn program(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Container => text(row.item, "exe").to_string(),
        Kind::Socket => UNSAID.to_string(),
    }
}

pub fn host_paths(item: &Value) -> Vec<&str> {
    item["host_paths"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn mounted(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Container => match mounts_readable(row.item) {
            true => host_paths(row.item).len().to_string(),
            false => "?".to_string(),
        },
        Kind::Socket => UNSAID.to_string(),
    }
}

pub fn mounts_readable(item: &Value) -> bool {
    item["mounts_readable"].as_bool().unwrap_or(true)
}

pub fn truncated(item: &Value) -> bool {
    item["host_paths_truncated"].as_bool().unwrap_or(false)
}

pub fn capabilities(item: &Value) -> Option<u64> {
    u64::from_str_radix(item["capabilities_effective"].as_str()?, 16).ok()
}

pub fn may_take_the_host(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Socket => UNSAID.to_string(),
        Kind::Container => match capabilities(row.item) {
            Some(held) if held & SYS_ADMIN != 0 => "yes".to_string(),
            Some(_) => "no".to_string(),
            None => "?".to_string(),
        },
    }
}

pub fn identity(row: &Row<'_>) -> String {
    match row.item["exe"].as_str() {
        Some(exe) if !exe.is_empty() => exe.to_string(),
        _ => short(text(row.item, "id")).to_string(),
    }
}

pub fn sort_key(row: &Row<'_>) -> (Kind, String) {
    (row.kind, what(row))
}

pub fn short(id: &str) -> &str {
    match id.char_indices().nth(12) {
        Some((at, _)) => &id[..at],
        None => id,
    }
}

pub fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap_or("?")
}

pub fn means(row: &Row<'_>) -> Vec<String> {
    let mut said = Vec::new();
    if capabilities(row.item).is_some_and(|held| held & SYS_ADMIN != 0) {
        said.push(
            "This container holds SYS_ADMIN, the capability that mounts filesystems and loads \
             what the kernel will take: from inside it, this host can be left."
                .to_string(),
        );
    }
    if host_paths(row.item)
        .iter()
        .any(|path| path.ends_with(".sock"))
    {
        said.push(
            "One of the paths it holds is a socket: if it is the socket of a container \
             runtime, whatever writes to it can start a container as root on this host."
                .to_string(),
        );
    }
    if !mounts_readable(row.item) {
        said.push(
            "The paths this container has mounted of this host could not be read, so what it \
             holds of the host is unknown rather than nothing."
                .to_string(),
        );
    }
    if truncated(row.item) {
        said.push(
            "This container mounts more host paths than the reading carries, and the list of \
             them is cut."
                .to_string(),
        );
    }
    said
}
