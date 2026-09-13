use serde_json::Value;
use vigil_view::bytes;

use crate::types::Family;

const UNSAID: &str = "—";

const SETUID: u32 = 0o4000;

const SETGID: u32 = 0o2000;

const WRITABLE_BY_ANYONE: u32 = 0o0002;

const STICKY: u32 = 0o1000;

pub fn what(item: &Value) -> String {
    text(item, "path").to_string()
}

pub fn mode(item: &Value) -> String {
    item["mode"].as_str().unwrap_or(UNSAID).to_string()
}

pub fn owner(item: &Value) -> String {
    match (item["uid"].as_u64(), item["gid"].as_u64()) {
        (Some(uid), Some(gid)) => format!("{uid}:{gid}"),
        _ => UNSAID.to_string(),
    }
}

pub fn held(family: Family, item: &Value) -> String {
    match family {
        Family::Directory => UNSAID.to_string(),
        Family::File => match present(item) {
            true => bytes(item["size"].as_u64().unwrap_or_default()),
            false => UNSAID.to_string(),
        },
    }
}

pub fn standing(family: Family, item: &Value) -> String {
    if !present(item) {
        return "not there".to_string();
    }
    if family == Family::File && !readable(item) {
        return "unreadable".to_string();
    }
    if over_the_ceiling(item) {
        return "too big to hash".to_string();
    }
    match marks(item) {
        marks if marks.is_empty() => "ordinary".to_string(),
        marks => marks.join(" · "),
    }
}

pub fn marks(item: &Value) -> Vec<&'static str> {
    let Some(bits) = bits(item) else {
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

pub fn digest(item: &Value) -> String {
    match item["sha256"].as_str() {
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

pub fn sort_key(family: Family, item: &Value) -> (Family, String) {
    (family, what(item))
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

pub fn means(family: Family, item: &Value) -> Vec<String> {
    if !present(item) {
        return vec![
            "This path is not on the host. A watched path that is not there is watched all \
             the same, and its appearing is a change."
                .to_string(),
        ];
    }

    let mut said = Vec::new();
    if family == Family::File && !readable(item) {
        said.push(
            "This file is on the host and this agent may not read it, so no hash of it is \
             held and a change to what it holds would pass unseen."
                .to_string(),
        );
    }
    if over_the_ceiling(item) {
        said.push(
            "This file is past the size this agent hashes, so it is watched by its \
             permissions and its owner alone."
                .to_string(),
        );
    }
    said
}
