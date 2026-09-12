use serde_json::Value;

use super::kind::Kind;
use super::row::Row;
use crate::ui::helpers::words::size;

const UNSAID: &str = "—";

const SETUID: u32 = 0o4000;

const SETGID: u32 = 0o2000;

const WRITABLE_BY_ANYONE: u32 = 0o0002;

const STICKY: u32 = 0o1000;

pub fn what(row: &Row<'_>) -> String {
    text(row.item, "path").to_string()
}

pub fn mode(row: &Row<'_>) -> String {
    row.item["mode"].as_str().unwrap_or(UNSAID).to_string()
}

pub fn owner(row: &Row<'_>) -> String {
    match (row.item["uid"].as_u64(), row.item["gid"].as_u64()) {
        (Some(uid), Some(gid)) => format!("{uid}:{gid}"),
        _ => UNSAID.to_string(),
    }
}

pub fn held(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Directory => UNSAID.to_string(),
        Kind::File => match present(row.item) {
            true => size::bytes(row.item["size"].as_u64().unwrap_or_default()),
            false => UNSAID.to_string(),
        },
    }
}

pub fn standing(row: &Row<'_>) -> String {
    if !present(row.item) {
        return "not there".to_string();
    }
    if row.kind == Kind::File && !readable(row.item) {
        return "unreadable".to_string();
    }
    if over_the_ceiling(row.item) {
        return "too big to hash".to_string();
    }
    match marks(row) {
        marks if marks.is_empty() => "ordinary".to_string(),
        marks => marks.join(" · "),
    }
}

pub fn marks(row: &Row<'_>) -> Vec<&'static str> {
    let Some(bits) = bits(row.item) else {
        return Vec::new();
    };
    let mut marks = Vec::new();
    if bits & SETUID != 0 {
        marks.push("suid");
    }
    if bits & SETGID != 0 {
        marks.push("sgid");
    }
    if bits & WRITABLE_BY_ANYONE != 0 && bits & STICKY == 0 {
        marks.push("anyone may write");
    }
    marks
}

pub fn digest(row: &Row<'_>) -> String {
    match row.item["sha256"].as_str() {
        Some(digest) => short(digest).to_string(),
        None => UNSAID.to_string(),
    }
}

pub fn present(item: &Value) -> bool {
    item["present"].as_bool().unwrap_or(false)
}

pub fn readable(item: &Value) -> bool {
    item["readable"].as_bool().unwrap_or(true)
}

pub fn over_the_ceiling(item: &Value) -> bool {
    item["over_the_ceiling"].as_bool().unwrap_or(false)
}

pub fn bits(item: &Value) -> Option<u32> {
    u32::from_str_radix(item["mode"].as_str()?, 8).ok()
}

pub fn sort_key(row: &Row<'_>) -> (Kind, String) {
    (row.kind, what(row))
}

pub fn short(digest: &str) -> &str {
    match digest.char_indices().nth(12) {
        Some((at, _)) => &digest[..at],
        None => digest,
    }
}

pub fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap_or("?")
}

pub fn means(row: &Row<'_>) -> Vec<String> {
    if !present(row.item) {
        return vec![
            "This path is not on the host. A watched path that is not there is watched all \
             the same, and its appearing is a change."
                .to_string(),
        ];
    }

    let mut said = Vec::new();
    if row.kind == Kind::File && !readable(row.item) {
        said.push(
            "This file is on the host and this agent may not read it, so no hash of it is \
             held and a change to what it holds would pass unseen."
                .to_string(),
        );
    }
    if over_the_ceiling(row.item) {
        said.push(
            "This file is past the size this agent hashes, so it is watched by its \
             permissions and its owner alone."
                .to_string(),
        );
    }
    said
}
