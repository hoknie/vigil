use serde_json::Value;

use crate::types::Kind;

const UNSAID: &str = "—";

pub const SYS_ADMIN: u64 = 1 << 21;

pub fn what(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Container) {
        Kind::Container => short(text(item, "id")).to_string(),
        Kind::Socket => text(item, "path").to_string(),
    }
}

pub fn runtime(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Container) {
        Kind::Container => text(item, "runtime").to_string(),
        Kind::Socket => UNSAID.to_string(),
    }
}

pub fn program(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Container) {
        Kind::Container => text(item, "exe").to_string(),
        Kind::Socket => UNSAID.to_string(),
    }
}

pub fn host_paths(item: &Value) -> Vec<&str> {
    item["host_paths"]
        .as_array()
        .map(|values| values.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

pub fn mounted(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Container) {
        Kind::Container => match mounts_readable(item) {
            true => host_paths(item).len().to_string(),
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

pub fn may_take_the_host(key: &str, item: &Value) -> String {
    match Kind::of(key).unwrap_or(Kind::Container) {
        Kind::Socket => UNSAID.to_string(),
        Kind::Container => match capabilities(item) {
            Some(held) if held & SYS_ADMIN != 0 => "yes".to_string(),
            Some(_) => "no".to_string(),
            None => "?".to_string(),
        },
    }
}

pub fn identity(item: &Value) -> String {
    match item["exe"].as_str() {
        Some(exe) if !exe.is_empty() => exe.to_string(),
        _ => short(text(item, "id")).to_string(),
    }
}

pub fn sort_key(key: &str, item: &Value) -> (Kind, String) {
    (Kind::of(key).unwrap_or(Kind::Container), what(key, item))
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

pub fn means(item: &Value) -> Vec<String> {
    let mut said = Vec::new();
    if capabilities(item).is_some_and(|held| held & SYS_ADMIN != 0) {
        said.push(
            "This container holds SYS_ADMIN, the capability that mounts filesystems and loads \
             what the kernel will take: from inside it, this host can be left."
                .to_string(),
        );
    }
    if host_paths(item).iter().any(|path| path.ends_with(".sock")) {
        said.push(
            "One of the paths it holds is a socket: if it is the socket of a container \
             runtime, whatever writes to it can start a container as root on this host."
                .to_string(),
        );
    }
    if !mounts_readable(item) {
        said.push(
            "The paths this container has mounted of this host could not be read, so what it \
             holds of the host is unknown rather than nothing."
                .to_string(),
        );
    }
    if truncated(item) {
        said.push(
            "This container mounts more host paths than the reading carries, and the list of \
             them is cut."
                .to_string(),
        );
    }
    said
}
