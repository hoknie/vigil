use std::collections::{BTreeMap, BTreeSet};

use super::session::{LOGIND, Session};

const NAMES: &[&str] = &["USER", "NAME"];

pub fn parse_logind_session(id: &str, text: &str) -> Option<Session> {
    let fields = fields(text);
    if !NAMES.iter().any(|name| fields.contains_key(*name)) && !fields.contains_key("UID") {
        return None;
    }

    let from = value(&fields, &["REMOTE_HOST"]);
    Some(Session {
        user: value(&fields, NAMES),
        uid: fields.get("UID").and_then(|uid| uid.parse().ok()),
        line: value(&fields, &["TTY"]),
        remote: fields.get("REMOTE").map(String::as_str) == Some("1") || !from.is_empty(),
        from,
        pid: fields
            .get("LEADER")
            .and_then(|leader| leader.parse().ok())
            .unwrap_or(0),
        id: id.to_string(),
        service: value(&fields, &["SERVICE"]),
        kind: value(&fields, &["TYPE"]),
        class: value(&fields, &["CLASS"]),
        state: value(&fields, &["STATE"]),
        sources: BTreeSet::from([LOGIND]),
    })
}

pub fn is_session_file(name: &str) -> bool {
    !name.is_empty() && !name.starts_with('.') && !name.contains('.')
}

fn fields(text: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((name, value)) = line.split_once('=') else {
            continue;
        };
        fields.insert(
            name.trim().to_string(),
            value.trim().trim_matches('"').to_string(),
        );
    }
    fields
}

fn value(fields: &BTreeMap<String, String>, names: &[&str]) -> String {
    names
        .iter()
        .find_map(|name| fields.get(*name))
        .cloned()
        .unwrap_or_default()
}
