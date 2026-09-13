use std::collections::BTreeMap;

use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Emphasis, RowKey, Showing};

use super::tree::{Placed, tree};
use crate::types::{Kind, List};

pub(super) const TREE: &str = "tree";

pub(super) fn rows(reading: &Snapshot, list: List, showing: &Showing<'_>) -> Vec<RowKey> {
    let mut rows: Vec<RowKey> = match nested(list, showing) {
        true => tree(&reading.items).into_iter().map(row_of).collect(),
        false => {
            let mut flat: Vec<(bool, String)> = reading
                .items
                .keys()
                .filter(|key| List::holding(key) == list)
                .map(|key| (!Kind::of(key).mark(), key.clone()))
                .collect();
            flat.sort();
            flat.into_iter()
                .map(|(_, key)| said(RowKey::of(key)))
                .collect()
        }
    };

    rows.retain(|row| {
        reading
            .items
            .get(&row.key)
            .is_some_and(|item| showing.matches(&row.key, item))
    });
    rows
}

pub(super) fn nested(list: List, showing: &Showing<'_>) -> bool {
    list == List::Units && showing.arranged_as(TREE)
}

pub(super) fn parents_of(items: &BTreeMap<String, Value>, key: &str) -> usize {
    tree(items)
        .into_iter()
        .find(|placed| placed.key == key)
        .map(|placed| placed.parents)
        .unwrap_or_default()
}

fn row_of(placed: Placed) -> RowKey {
    said(RowKey::of(placed.key).under(placed.depth.min(u8::MAX as usize) as u8))
}

fn said(row: RowKey) -> RowKey {
    match Kind::of(&row.key).mark() {
        true => row.said(Emphasis::Marked),
        false => row,
    }
}
