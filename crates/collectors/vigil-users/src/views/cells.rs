use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::Cell;

use super::columns::where_from;
use super::facts::{group_named, keys_of, password, readable, route_to_root, seen_by, what};
use super::fields::{members, number, rules, text};
use crate::types::{Kind, Subject};

pub(super) fn cells(
    subject: Subject,
    key: &str,
    item: &Value,
    reading: &Snapshot,
    wide: bool,
) -> Vec<Cell> {
    let mut cells = match subject {
        Subject::Users => vec![
            name_of(key, item),
            number(item, "uid"),
            password(item),
            text(item, "shell").unwrap_or("?").to_string(),
            match route_to_root(reading, item).as_str() {
                "" => "—".to_string(),
                routes => routes.to_string(),
            },
        ],
        Subject::Groups => vec![
            name_of(key, item),
            number(item, "gid"),
            match text(item, "privilege") {
                Some(why) => why.to_string(),
                None => "—".to_string(),
            },
            match members(item).collect::<Vec<_>>().join(", ") {
                empty if empty.is_empty() => "—".to_string(),
                members => members,
            },
        ],
        Subject::Sudo => vec![
            name_of(key, item),
            match item.get("all_commands").and_then(Value::as_bool) == Some(true) {
                true => "every command".to_string(),
                false => "some commands".to_string(),
            },
            match item.get("nopasswd").and_then(Value::as_bool) == Some(true) {
                true => "not asked".to_string(),
                false => "asked for".to_string(),
            },
            reaches(reading, item),
        ],
        Subject::Keys => match readable(item) {
            false => vec![
                name_of(key, item),
                "—".to_string(),
                "this account's key file was not readable".to_string(),
                "—".to_string(),
            ],
            true => vec![
                name_of(key, item),
                text(item, "algorithm").unwrap_or("?").to_string(),
                text(item, "fingerprint").unwrap_or("?").to_string(),
                match text(item, "options") {
                    Some(_) => "restricted".to_string(),
                    None => "unrestricted".to_string(),
                },
            ],
        },
        Subject::SshUsers => vec![
            name_of(key, item),
            number(item, "uid"),
            key_count(reading, &name_of(key, item)),
            match Kind::of(key) {
                Kind::Account => text(item, "shell").unwrap_or("?").to_string(),
                _ => "not in /etc/passwd".to_string(),
            },
        ],
        Subject::LoggedIn if Kind::of(key) == Kind::SessionSource => vec![
            name_of(key, item),
            "—".to_string(),
            text(item, "path").unwrap_or("?").to_string(),
            format!("login source, {}", standing(item)),
            name_of(key, item),
        ],
        Subject::LoggedIn => vec![
            name_of(key, item),
            match text(item, "line") {
                Some(line) if !line.is_empty() => line.to_string(),
                _ => "—".to_string(),
            },
            match text(item, "from") {
                Some(from) if !from.is_empty() => from.to_string(),
                _ => "this host's console".to_string(),
            },
            what(item),
            seen_by(item),
        ],
        Subject::Other => vec![
            key.to_string(),
            "a kind of object this console does not know".to_string(),
        ],
    };

    if wide && where_from(subject).is_some() {
        cells.push(source(subject, key, item, reading));
    }

    cells.into_iter().map(Cell::plain).collect()
}

fn source(subject: Subject, key: &str, item: &Value, reading: &Snapshot) -> String {
    match subject {
        Subject::Users => text(item, "home").unwrap_or("—").to_string(),
        Subject::Sudo => rules(item)
            .filter_map(|rule| text(rule, "source"))
            .next()
            .unwrap_or("?")
            .to_string(),
        Subject::Keys => text(item, "source").unwrap_or("—").to_string(),
        Subject::SshUsers => keys_of(reading, &name_of(key, item))
            .filter_map(|(_, item)| text(item, "source"))
            .next()
            .unwrap_or("—")
            .to_string(),
        Subject::Other => key.split('|').next().unwrap_or("?").to_string(),
        Subject::LoggedIn => match Kind::of(key) {
            Kind::SessionSource => "—".to_string(),
            _ => number(item, "pid"),
        },
        Subject::Groups => String::new(),
    }
}

fn key_count(reading: &Snapshot, user: &str) -> String {
    let mut readable_keys = 0;
    let mut refused = false;
    for (_, item) in keys_of(reading, user) {
        match readable(item) {
            true => readable_keys += 1,
            false => refused = true,
        }
    }
    match (refused, readable_keys) {
        (true, 0) => "refused".to_string(),
        (true, count) => format!("{count} + refused"),
        (false, count) => count.to_string(),
    }
}

fn reaches(reading: &Snapshot, item: &Value) -> String {
    let Some(who) = text(item, "who") else {
        return "?".to_string();
    };
    match who.strip_prefix('%') {
        None => who.to_string(),
        Some(group) => {
            let members: Vec<&str> = group_named(reading, group)
                .map(|found| members(found).collect())
                .unwrap_or_default();
            match members.is_empty() {
                true => format!("{who}: nobody is in it"),
                false => format!("{who}: {}", members.join(", ")),
            }
        }
    }
}

fn standing(source: &Value) -> String {
    match (
        source.get("present").and_then(Value::as_bool),
        source.get("read").and_then(Value::as_bool),
    ) {
        (Some(false), _) => "not on this host".to_string(),
        (_, Some(false)) => "on this host and not read".to_string(),
        _ => match number(source, "sessions").as_str() {
            "1" => "read: 1 session".to_string(),
            many => format!("read: {many} sessions"),
        },
    }
}

pub(super) fn name_of(key: &str, item: &Value) -> String {
    match Kind::of(key) {
        Kind::SessionSource => text(item, "source").unwrap_or("?").to_string(),
        Kind::Sudoer => text(item, "who").unwrap_or("?").to_string(),
        Kind::Key | Kind::Session => text(item, "user").unwrap_or("?").to_string(),
        _ => text(item, "name")
            .map(str::to_string)
            .unwrap_or_else(|| key.to_string()),
    }
}
