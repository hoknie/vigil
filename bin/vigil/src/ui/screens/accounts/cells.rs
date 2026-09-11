use ratatui::widgets::Row as TableRow;
use serde_json::Value;

use super::columns::where_from;
use super::facts::{password, readable, route_to_root, seen_by, what};
use super::fields::{members, number, objects, rules, text};
use super::kind::Kind;
use super::row::Row;
use super::rows::keys_of;
use crate::ui::helpers::layout::column;
use crate::ui::{Subject, View};

pub(super) fn cells(
    subject: Subject,
    row: &Row<'_>,
    view: &View,
    wide: bool,
    columns: &[usize],
) -> TableRow<'static> {
    let mut cells = match subject {
        Subject::Users => vec![
            name(row),
            number(row.item, "uid"),
            password(row.item),
            text(row.item, "shell").unwrap_or("?").to_string(),
            match route_to_root(view, row.item).as_str() {
                "" => "—".to_string(),
                routes => routes.to_string(),
            },
        ],
        Subject::Groups => vec![
            name(row),
            number(row.item, "gid"),
            match text(row.item, "privilege") {
                Some(why) => why.to_string(),
                None => "—".to_string(),
            },
            match members(row.item).collect::<Vec<_>>().join(", ") {
                empty if empty.is_empty() => "—".to_string(),
                members => members,
            },
        ],
        Subject::Sudo => vec![
            name(row),
            match row.item.get("all_commands").and_then(Value::as_bool) == Some(true) {
                true => "every command".to_string(),
                false => "some commands".to_string(),
            },
            match row.item.get("nopasswd").and_then(Value::as_bool) == Some(true) {
                true => "not asked".to_string(),
                false => "asked for".to_string(),
            },
            reaches(view, row),
        ],
        Subject::Keys => match readable(row.item) {
            false => vec![
                name(row),
                "—".to_string(),
                "this account's key file was not readable".to_string(),
                "—".to_string(),
            ],
            true => vec![
                name(row),
                text(row.item, "algorithm").unwrap_or("?").to_string(),
                text(row.item, "fingerprint").unwrap_or("?").to_string(),
                match text(row.item, "options") {
                    Some(_) => "restricted".to_string(),
                    None => "unrestricted".to_string(),
                },
            ],
        },
        Subject::SshUsers => vec![
            name(row),
            number(row.item, "uid"),
            key_count(view, &name(row)),
            match row.kind {
                Kind::Account => text(row.item, "shell").unwrap_or("?").to_string(),
                _ => "not in /etc/passwd".to_string(),
            },
        ],
        Subject::LoggedIn if row.kind == Kind::SessionSource => vec![
            name(row),
            "—".to_string(),
            text(row.item, "path").unwrap_or("?").to_string(),
            format!("login source, {}", standing(row.item)),
            name(row),
        ],
        Subject::LoggedIn => vec![
            name(row),
            match text(row.item, "line") {
                Some(line) if !line.is_empty() => line.to_string(),
                _ => "—".to_string(),
            },
            match text(row.item, "from") {
                Some(from) if !from.is_empty() => from.to_string(),
                _ => "this host's console".to_string(),
            },
            what(row.item),
            seen_by(row.item),
        ],
        Subject::Other => vec![
            row.key.clone(),
            "a kind of object this console does not know".to_string(),
        ],
    };

    if wide && where_from(subject).is_some() {
        cells.push(source(subject, row, view));
    }

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}

fn source(subject: Subject, row: &Row<'_>, view: &View) -> String {
    match subject {
        Subject::Users => text(row.item, "home").unwrap_or("—").to_string(),
        Subject::Sudo => rules(row.item)
            .filter_map(|rule| text(rule, "source"))
            .next()
            .unwrap_or("?")
            .to_string(),
        Subject::Keys => text(row.item, "source").unwrap_or("—").to_string(),
        Subject::SshUsers => keys_of(view, &name(row))
            .filter_map(|(_, item)| text(item, "source"))
            .next()
            .unwrap_or("—")
            .to_string(),
        Subject::Other => row.key.split('|').next().unwrap_or("?").to_string(),
        Subject::LoggedIn => match row.kind {
            Kind::SessionSource => "—".to_string(),
            _ => number(row.item, "pid"),
        },
        Subject::Groups => String::new(),
    }
}

fn key_count(view: &View, user: &str) -> String {
    let mut readable_keys = 0;
    let mut refused = false;
    for (_, item) in keys_of(view, user) {
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

fn reaches(view: &View, row: &Row<'_>) -> String {
    let Some(who) = text(row.item, "who") else {
        return "?".to_string();
    };
    match who.strip_prefix('%') {
        None => who.to_string(),
        Some(group) => {
            let members: Vec<&str> = objects(view, Kind::Group)
                .filter(|(_, item)| text(item, "name") == Some(group))
                .flat_map(|(_, item)| members(item))
                .collect();
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

pub(super) fn name(row: &Row<'_>) -> String {
    match row.kind {
        Kind::SessionSource => text(row.item, "source").unwrap_or("?").to_string(),
        Kind::Sudoer => text(row.item, "who").unwrap_or("?").to_string(),
        Kind::Key | Kind::Session => text(row.item, "user").unwrap_or("?").to_string(),
        _ => text(row.item, "name")
            .map(str::to_string)
            .unwrap_or_else(|| row.key.clone()),
    }
}
