use serde_json::Value;

use super::cells::name;
use super::facts::{attended, could_log_in, privileged, readable, remote};
use super::fields::{objects, text};
use super::kind::Kind;
use super::row::Row;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, Search, Subject, View};

pub fn rows<'a>(view: &'a View, subject: Subject, search: &Search) -> Vec<Row<'a>> {
    if subject == Subject::SshUsers {
        return ssh_users(view, search);
    }
    let kinds = subject.kinds();
    if kinds.is_empty() {
        return Vec::new();
    }
    let Reading::Taken(snapshot) = view.reading("users") else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = snapshot
        .items
        .iter()
        .filter(|(key, _)| kinds.contains(&Kind::of(key)))
        .filter(|(key, item)| search.matches(&haystack::haystack(key, item)))
        .map(|(key, item)| Row {
            key: key.clone(),
            kind: Kind::of(key),
            item,
        })
        .collect();

    rows.sort_by_key(|row| (rank(row), row.key.clone()));
    rows
}

fn rank(row: &Row<'_>) -> u8 {
    match row.kind {
        Kind::Account => u8::from(!could_log_in(row.item)),
        Kind::Group => u8::from(!privileged(row.item)),
        Kind::Key => u8::from(readable(row.item)),
        Kind::Session => match (attended(row.item), remote(row.item)) {
            (true, true) => 0,
            (true, false) => 1,
            (false, _) => 2,
        },
        Kind::SessionSource => 3,
        _ => 0,
    }
}

fn ssh_users<'a>(view: &'a View, search: &Search) -> Vec<Row<'a>> {
    let mut names: Vec<&str> = objects(view, Kind::Key)
        .filter_map(|(_, item)| text(item, "user"))
        .collect();
    names.sort_unstable();
    names.dedup();

    let mut rows: Vec<Row<'a>> = names
        .into_iter()
        .filter_map(|name| {
            let account = objects(view, Kind::Account)
                .find(|(_, item)| text(item, "name") == Some(name))
                .map(|(key, item)| Row {
                    key: key.to_string(),
                    kind: Kind::Account,
                    item,
                });
            account.or_else(|| {
                keys_of(view, name).next().map(|(key, item)| Row {
                    key: key.to_string(),
                    kind: Kind::Key,
                    item,
                })
            })
        })
        .filter(|row| {
            let mut haystack = haystack::haystack(&row.key, row.item);
            for (key, item) in keys_of(view, &name(row)) {
                haystack.push(' ');
                haystack.push_str(&haystack::haystack(key, item));
            }
            search.matches(&haystack)
        })
        .collect();

    rows.sort_by_key(|row| {
        let name = name(row);
        let read_them_all = keys_of(view, &name).all(|(_, item)| readable(item));
        (read_them_all, name)
    });
    rows
}

pub(super) fn keys_of<'a>(
    view: &'a View,
    user: &str,
) -> impl Iterator<Item = (&'a str, &'a Value)> {
    let user = user.to_string();
    objects(view, Kind::Key).filter(move |(_, item)| text(item, "user") == Some(user.as_str()))
}

pub fn keys(view: &View, subject: Subject, search: &Search) -> Vec<String> {
    rows(view, subject, search)
        .into_iter()
        .map(|row| row.key)
        .collect()
}
