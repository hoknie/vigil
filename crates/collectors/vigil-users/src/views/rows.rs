use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{RowKey, Showing, haystack};

use super::facts::{attended, could_log_in, privileged, readable, remote};
use super::fields::{objects, text};
use crate::types::{Kind, Subject};

pub(super) fn rows(reading: &Snapshot, subject: Subject, showing: &Showing<'_>) -> Vec<RowKey> {
    if subject == Subject::SshUsers {
        return ssh_users(reading, showing);
    }
    let kinds = subject.kinds();
    if kinds.is_empty() {
        return Vec::new();
    }

    let mut rows: Vec<(u8, String)> = reading
        .items
        .iter()
        .filter(|(key, _)| kinds.contains(&Kind::of(key)))
        .filter(|(key, item)| showing.matches(key, item))
        .map(|(key, item)| (rank(Kind::of(key), item), key.clone()))
        .collect();

    rows.sort();
    rows.into_iter().map(|(_, key)| RowKey::of(key)).collect()
}

fn rank(kind: Kind, item: &Value) -> u8 {
    match kind {
        Kind::Account => u8::from(!could_log_in(item)),
        Kind::Group => u8::from(!privileged(item)),
        Kind::Key => u8::from(readable(item)),
        Kind::Session => match (attended(item), remote(item)) {
            (true, true) => 0,
            (true, false) => 1,
            (false, _) => 2,
        },
        Kind::SessionSource => 3,
        _ => 0,
    }
}

fn ssh_users(reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
    let mut names: Vec<&str> = objects(reading, Kind::Key)
        .filter_map(|(_, item)| text(item, "user"))
        .collect();
    names.sort_unstable();
    names.dedup();

    let mut rows: Vec<(bool, String, String)> = names
        .into_iter()
        .filter_map(|name| {
            let account = objects(reading, Kind::Account)
                .find(|(_, item)| text(item, "name") == Some(name))
                .map(|(key, _)| key.to_string());
            let key = account.or_else(|| {
                keys_of(reading, name)
                    .next()
                    .map(|(key, _)| key.to_string())
            })?;
            Some((name.to_string(), key))
        })
        .filter(|(name, key)| {
            let Some(item) = reading.items.get(key) else {
                return false;
            };
            let mut searched = haystack(key, item);
            for (other, item) in keys_of(reading, name) {
                searched.push(' ');
                searched.push_str(&haystack(other, item));
            }
            showing.search.is_empty()
                || searched
                    .to_lowercase()
                    .contains(&showing.search.to_lowercase())
        })
        .map(|(name, key)| {
            let read_them_all = keys_of(reading, &name).all(|(_, item)| readable(item));
            (read_them_all, name, key)
        })
        .collect();

    rows.sort();
    rows.into_iter()
        .map(|(_, _, key)| RowKey::of(key))
        .collect()
}

pub(super) fn keys_of<'a>(
    reading: &'a Snapshot,
    user: &str,
) -> impl Iterator<Item = (&'a str, &'a Value)> {
    let user = user.to_string();
    objects(reading, Kind::Key).filter(move |(_, item)| text(item, "user") == Some(user.as_str()))
}
