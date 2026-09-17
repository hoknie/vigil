use serde_json::Value;
use vigil_model::{Changing, Snapshot};
use vigil_view::{
    Cell, Column, Counts, Form, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows, Showing,
    Sorting, Width, every_field, haystack,
};

use super::fields::{digest, held, means, mode, owner, sort_key, standing, what};
use super::footer::footer;
use super::form::form;
use super::notices;
use super::tally::tally;
use crate::types::Family;

const ROOM_FOR_THE_HASH: u16 = 118;

const SORTED_BY: &[&str] = &["KIND", "PATH", "MODE", "OWNER", "SIZE", "STANDING"];

pub(super) struct WatchedFiles;

impl Pane for WatchedFiles {
    fn name(&self) -> &str {
        "watched files"
    }

    fn caption(&self) -> &str {
        "WATCHED FILES"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED PATH"
    }

    fn about(&self) -> &str {
        "the files this host is configured by, and the directories on PATH a program could be \
         dropped into"
    }

    fn reads(&self) -> &str {
        "files"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let wide = room.holds(ROOM_FOR_THE_HASH);
        let mut columns = vec![
            Column::new("KIND", Width::Fixed(9)),
            Column::new("PATH", Width::Share(1)),
            Column::new("MODE", Width::Fixed(5)),
            Column::new("OWNER", Width::Fixed(7)),
        ];
        if wide {
            columns.push(Column::new("SIZE", Width::Fixed(9)));
            columns.push(Column::new("SHA256", Width::Fixed(13)));
        }
        columns.push(Column::new("STANDING", Width::Fixed(17)));
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<((Family, String), String)> = reading
            .items
            .iter()
            .filter_map(|(key, item)| Family::of(key).map(|family| (family, key, item)))
            .filter(|(_, key, item)| showing.matches(key, item))
            .map(|(family, key, item)| (sort_key(family, item), key.clone()))
            .collect();

        rows.sort();
        let mut keys: Vec<String> = rows.into_iter().map(|(_, key)| key).collect();
        sort(&mut keys, reading, showing.sorting);

        keys.into_iter().map(RowKey::of).collect()
    }

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        let mut read: Vec<((Family, String), &String, &Value)> = reading
            .items
            .iter()
            .filter_map(|(key, item)| {
                Family::of(key).map(|family| (sort_key(family, item), key, item))
            })
            .collect();
        read.sort_by(|left, right| (&left.0, left.1).cmp(&(&right.0, right.1)));

        let mut index = Index::new(SORTED_BY.len());
        for (_, key, item) in read {
            let keys = (1..=SORTED_BY.len())
                .map(|by| sorted_on(reading, key, by))
                .collect();
            index.push(RowKey::of(key.as_str()), &haystack(key, item), 0, keys);
        }
        Some(index)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let (Some(family), Some(item)) = (Family::of(&row.key), reading.items.get(&row.key)) else {
            return Vec::new();
        };

        let mut cells = vec![
            Cell::plain(family.name()),
            Cell::plain(what(item)),
            Cell::plain(mode(item)),
            Cell::plain(owner(item)),
        ];
        if room.holds(ROOM_FOR_THE_HASH) {
            cells.push(Cell::plain(held(family, item)));
            cells.push(Cell::plain(digest(item)));
        }
        cells.push(Cell::plain(standing(family, item)));

        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let (Some(family), Some(item)) = (Family::of(&row.key), reading.items.get(&row.key)) else {
            return Vec::new();
        };

        every_field(
            family.name(),
            &what(item),
            &row.key,
            item,
            &means(family, item),
        )
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown)
    }

    fn counts(&self, _reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        Some(Counts::default())
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        _counts: &Counts,
    ) -> String {
        footer(reading, showing, rows)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(showing)
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(notices::nothing_read())
    }

    fn nothing_was_read(&self) -> &'static str {
        "No file and no directory is listed here: nothing was read."
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn offers(&self) -> Offers {
        Offers::default().watched(true)
    }

    fn form(
        &self,
        reading: &Snapshot,
        row: Option<&RowKey>,
        changing: Changing,
    ) -> Result<Form, String> {
        form(reading, row, changing)
    }
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
    let (Some(family), Some(item)) = (Family::of(key), reading.items.get(key)) else {
        return String::new();
    };
    match by {
        1 => family.name().to_string(),
        2 => what(item),
        3 => mode(item),
        4 => owner(item),
        5 => format!("{:0>20}", item["size"].as_u64().unwrap_or_default()),
        6 => standing(family, item),
        _ => String::new(),
    }
}
