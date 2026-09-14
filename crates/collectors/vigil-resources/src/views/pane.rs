use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Sorting,
    Width, every_field, haystack,
};

use super::fields::{
    device, free, free_inodes, free_inodes_step, free_step, held, how_it_is_mounted, kind_of_store,
    means, number, sort_key, what,
};
use super::footer::footer;
use super::notices;
use super::tally::tally;
use crate::types::Family;

const ROOM_FOR_THE_DETAIL: u16 = 118;

const SORTED_BY: &[&str] = &["KIND", "WHAT", "FREE", "INODES", "SIZE"];

pub(super) struct TheHost;

impl Pane for TheHost {
    fn name(&self) -> &str {
        "the host"
    }

    fn caption(&self) -> &str {
        "THE HOST"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED PART"
    }

    fn about(&self) -> &str {
        "the boot this host is running, the memory it has and the filesystems it holds its \
         files on"
    }

    fn reads(&self) -> &str {
        "resources"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let wide = room.holds(ROOM_FOR_THE_DETAIL);
        let mut columns = vec![
            Column::new("KIND", Width::Fixed(10)),
            Column::new("WHAT", Width::Share(1)),
        ];
        if wide {
            columns.push(Column::new("DETAIL", Width::Fixed(23)));
            columns.push(Column::new("DEVICE", Width::Fixed(12)));
        }
        columns.push(Column::new("FREE", Width::Fixed(6)));
        columns.push(Column::new("INODES", Width::Fixed(6)));
        columns.push(Column::new("SIZE", Width::Fixed(9)));
        if wide {
            columns.push(Column::new("MOUNTED", Width::Fixed(10)));
        }
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
        let wide = room.holds(ROOM_FOR_THE_DETAIL);

        let mut cells = vec![Cell::plain(family.name()), Cell::plain(what(family, item))];
        if wide {
            cells.push(Cell::plain(kind_of_store(family, item)));
            cells.push(Cell::plain(device(family, item)));
        }
        cells.push(Cell::plain(free(family, item)));
        cells.push(Cell::plain(free_inodes(family, item)));
        cells.push(Cell::plain(held(family, item)));
        if wide {
            cells.push(Cell::plain(how_it_is_mounted(family, item)));
        }

        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let (Some(family), Some(item)) = (Family::of(&row.key), reading.items.get(&row.key)) else {
            return Vec::new();
        };

        every_field(
            family.name(),
            &what(family, item),
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
        rows: &[RowKey],
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
        "No boot, no memory and no filesystem is listed here: nothing was read."
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn offers(&self) -> Offers {
        Offers::default()
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
        2 => what(family, item),
        3 => padded(free_step(item)),
        4 => padded(free_inodes_step(item)),
        5 => format!("{:0>20}", number(item, "total_bytes")),
        _ => String::new(),
    }
}

fn padded(step: Option<u64>) -> String {
    match step {
        Some(step) => format!("{step:0>3}"),
        None => String::new(),
    }
}
