use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Index, RowKey, haystack};

use super::facts::{keys_of, readable};
use super::fields::{objects, text};
use super::rows::rank;
use crate::types::{Kind, Subject};

const NO_SORTED_COLUMNS: usize = 0;

pub(super) fn index(reading: &Snapshot, subject: Subject) -> Index {
    let mut index = Index::new(NO_SORTED_COLUMNS);

    if subject == Subject::SshUsers {
        for (read_them_all, _, key, searched) in ssh_users(reading) {
            index.push(
                RowKey::of(key.to_string()),
                &searched,
                u8::from(read_them_all),
                Vec::new(),
            );
        }
        return index;
    }

    let mut rows: Vec<(u8, &str, &Value)> = subject
        .kinds()
        .iter()
        .flat_map(|kind| {
            objects(reading, *kind).map(move |(key, item)| (rank(*kind, item), key, item))
        })
        .collect();
    rows.sort_unstable_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));

    for (rank, key, item) in rows {
        index.push(
            RowKey::of(key.to_string()),
            &haystack(key, item),
            rank,
            Vec::new(),
        );
    }
    index
}

fn ssh_users(reading: &Snapshot) -> Vec<(bool, &str, &str, String)> {
    let mut names: Vec<&str> = objects(reading, Kind::Key)
        .filter_map(|(_, item)| text(item, "user"))
        .collect();
    names.sort_unstable();
    names.dedup();

    let mut rows: Vec<(bool, &str, &str, String)> = names
        .into_iter()
        .filter_map(|name| {
            let (key, item) = reading
                .items
                .get_key_value(&format!("{}{name}", Kind::Account.prefix()))
                .filter(|(_, item)| text(item, "name") == Some(name))
                .map(|(key, item)| (key.as_str(), item))
                .or_else(|| keys_of(reading, name).next())?;

            let mut searched = haystack(key, item);
            let mut read_them_all = true;
            for (other, found) in keys_of(reading, name) {
                searched.push(' ');
                searched.push_str(&haystack(other, found));
                read_them_all &= readable(found);
            }
            Some((read_them_all, name, key, searched))
        })
        .collect();

    rows.sort_unstable_by(|left, right| (left.0, left.1, left.2).cmp(&(right.0, right.1, right.2)));
    rows
}
