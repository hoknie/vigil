use vigil_model::Snapshot;
use vigil_view::{Index, RowKey, Showing, haystack};

use super::rows::{nested, row_of, said};
use super::tree::tree;
use crate::types::{Kind, List};

const NO_COLUMN: usize = 0;

const ONE_GROUP: u8 = 0;

pub(super) fn indexed(reading: &Snapshot, list: List, showing: &Showing<'_>) -> Index {
    let rows: Vec<RowKey> = match nested(list, showing) {
        true => tree(&reading.items).into_iter().map(row_of).collect(),
        false => {
            let mut flat: Vec<(bool, &String)> = reading
                .items
                .keys()
                .filter(|key| List::holding(key) == list)
                .map(|key| (!Kind::of(key).mark(), key))
                .collect();
            flat.sort();
            flat.into_iter()
                .map(|(_, key)| said(RowKey::of(key.as_str())))
                .collect()
        }
    };

    let mut index = Index::new(NO_COLUMN);
    for row in rows {
        if let Some(item) = reading.items.get(&row.key) {
            let text = haystack(&row.key, item);
            index.push(row, &text, ONE_GROUP, Vec::new());
        }
    }
    index
}
