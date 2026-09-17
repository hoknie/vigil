use std::ops::Bound;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Index, RowKey, Showing, Sorting, haystack};

use super::fields::{self, sort_key};
use crate::types::Kind;

pub(super) fn rows(reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
    let mut rows: Vec<((Kind, String, String), String)> = reading
        .items
        .iter()
        .filter(|(key, _)| of_the_ruleset(key))
        .filter(|(key, item)| showing.matches(key, item))
        .map(|(key, item)| (sort_key(key, item), key.clone()))
        .collect();

    rows.sort();
    let mut keys: Vec<String> = rows.into_iter().map(|(_, key)| key).collect();
    sort(&mut keys, reading, showing.sorting);

    keys.into_iter().map(RowKey::of).collect()
}

pub(super) fn indexed(reading: &Snapshot) -> Index {
    let mut read: Vec<((Kind, String, String), &String, &Value)> = reading
        .items
        .iter()
        .filter(|(key, _)| of_the_ruleset(key))
        .map(|(key, item)| (sort_key(key, item), key, item))
        .collect();
    read.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));

    let mut index = Index::new(SORTED_BY.len());
    for (_, key, item) in read {
        index.push(
            RowKey::of(key.clone()),
            &haystack(key, item),
            0,
            (1..=SORTED_BY.len())
                .map(|by| sorted_on(reading, key, by))
                .collect(),
        );
    }
    index
}

pub(super) fn summary(reading: &Snapshot) -> Option<&Value> {
    let word = Kind::Ruleset.word();
    reading
        .items
        .range::<str, _>((Bound::Included(word), Bound::Unbounded))
        .take_while(|(key, _)| key.starts_with(word))
        .find(|(key, _)| Kind::of(key) == Some(Kind::Ruleset))
        .map(|(_, item)| item)
}

pub(super) const SORTED_BY: &[&str] = &["KIND", "WHAT", "HOOK", "POLICY", "RULES"];

pub(super) fn of_the_ruleset(key: &str) -> bool {
    !matches!(Kind::of(key), None | Some(Kind::Interface))
}

fn sort(keys: &mut [String], reading: &Snapshot, sorting: Sorting) {
    if sorting.as_read() {
        return;
    }
    keys.sort_by(|left, right| {
        let ordering =
            sorted_on(reading, left, sorting.by).cmp(&sorted_on(reading, right, sorting.by));
        match sorting.descending {
            true => ordering.reverse(),
            false => ordering,
        }
    });
}

fn sorted_on(reading: &Snapshot, key: &str, by: usize) -> String {
    let Some(item) = reading.items.get(key) else {
        return String::new();
    };
    match by {
        1 => Kind::of(key).map_or(String::new(), |kind| kind.name().to_string()),
        2 => fields::what(key, item).to_lowercase(),
        3 => fields::hook(key, item),
        4 => fields::policy(key, item),
        5 => format!("{:0>12}", fields::rules(item)),
        _ => String::new(),
    }
}
