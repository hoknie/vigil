use serde_json::Value;
use vigil_view::bytes;

use crate::types::Family;

const UNSAID: &str = "—";

const SETUID: u32 = 0o4000;

const SETGID: u32 = 0o2000;

const WRITABLE_BY_ANYONE: u32 = 0o0002;

const STICKY: u32 = 0o1000;

pub fn what(item: &Value) -> String {
    match item["path"].as_str() {
        Some(path) => path.to_string(),
        None => text(item, "entry").to_string(),
    }
}

pub fn kind(family: Family, item: &Value) -> String {
    match family {
        Family::Walk => text(item, "kind").to_string(),
        Family::File => item["type"].as_str().unwrap_or("file").to_string(),
        Family::Directory => family.name().to_string(),
    }
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
        Family::Walk => format!("{} found", item["matched"].as_u64().unwrap_or_default()),
        Family::Directory => UNSAID.to_string(),
        Family::File if item["type"].as_str() == Some("directory") => UNSAID.to_string(),
        Family::File => match present(item) {
            true => bytes(item["size"].as_u64().unwrap_or_default()),
            false => UNSAID.to_string(),
        },
    }
}

pub fn standing(family: Family, item: &Value) -> String {
    if family == Family::Walk {
        return walked(item);
    }
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

fn walked(item: &Value) -> String {
    let not_entered = item["not_entered"].as_array().map_or(0, Vec::len);
    if item["complete"].as_bool() == Some(false) {
        return "cut at max_files".to_string();
    }
    if item["matched"].as_u64() == Some(0) {
        return match item["kind"].as_str() {
            Some("mask") => "matches nothing".to_string(),
            _ => "empty".to_string(),
        };
    }
    match not_entered {
        0 => "walked whole".to_string(),
        some => format!("{some} not entered"),
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
    if family == Family::Walk {
        return walk_means(item);
    }
    let mut said = found(item);
    if !present(item) {
        said.push(
            "This path is not on the host. A watched path that is not there is watched all \
             the same, and its appearing is a change."
                .to_string(),
        );
        return said;
    }

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

fn found(item: &Value) -> Vec<String> {
    let mut said = Vec::new();
    if let Some(by) = item["found_by"].as_str() {
        said.push(format!(
            "This path is watched because {by} is in the watch list: it is changed or stopped \
             there, and a path that appears under it or leaves it is a change."
        ));
    }
    if let Some(target) = item["target"].as_str() {
        said.push(format!(
            "This is a link to {target}, and it is not followed: where it points is what is \
             watched."
        ));
    }
    said
}

fn walk_means(item: &Value) -> Vec<String> {
    let entry = text(item, "entry");
    let matched = item["matched"].as_u64().unwrap_or_default();
    let mut said = vec![match text(item, "kind") {
        "mask" => format!("{entry} is a mask in the watch list, and {matched} path(s) match it."),
        _ => format!(
            "{entry} is a directory in the watch list, walked whole: {matched} path(s) under it."
        ),
    }];
    if matched == 0 && text(item, "kind") == "mask" {
        said.push(
            "A mask that matches nothing is not an error: the first path to match it is a \
             change."
                .to_string(),
        );
    }
    if item["complete"].as_bool() == Some(false) {
        said.push(
            "The walk stopped at max_files before it was done, so what lies past that point is \
             not watched."
                .to_string(),
        );
    }
    if let Some(skipped) = item["not_entered"]
        .as_array()
        .filter(|skipped| !skipped.is_empty())
    {
        let named: Vec<&str> = skipped.iter().filter_map(Value::as_str).collect();
        said.push(format!(
            "Not entered, by devices or because a walk never steps into the kernel's own \
             filesystems: {}.",
            named.join(", ")
        ));
    }
    said
}
