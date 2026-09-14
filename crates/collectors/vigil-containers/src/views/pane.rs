use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Sorting, Width, every_field,
};

use super::fields::{
    identity, may_take_the_host, means, mounted, program, runtime, sort_key, what,
};
use super::notices;
use super::tally::tally;
use crate::types::Kind;

const ROOM_FOR_THE_RUNTIME: u16 = 118;

const SORTED_BY: &[&str] = &["CONTAINER", "PROGRAM", "RUNTIME", "SYS_ADMIN", "HOST PATHS"];

pub(super) struct Contained;

impl Pane for Contained {
    fn name(&self) -> &str {
        "containers"
    }

    fn caption(&self) -> &str {
        "CONTAINERS AND SOCKETS"
    }

    fn about(&self) -> &str {
        "one row per container running on this host: what it may do, and what of this host it \
         holds"
    }

    fn reads(&self) -> &str {
        "containers"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("CONTAINER", Width::Fixed(14)),
            Column::new("PROGRAM", Width::Share(1)),
        ];
        if room.holds(ROOM_FOR_THE_RUNTIME) {
            columns.push(Column::new("RUNTIME", Width::Fixed(8)));
        }
        columns.push(Column::new("SYS_ADMIN", Width::Fixed(9)));
        columns.push(Column::new(
            match room.holds(ROOM_FOR_THE_RUNTIME) {
                true => "HOST PATHS",
                false => "PATHS",
            },
            Width::Fixed(10),
        ));
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<((Kind, String), String)> = reading
            .items
            .iter()
            .filter(|(key, _)| Kind::of(key) == Some(Kind::Container))
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, item)| (sort_key(key, item), key.clone()))
            .collect();

        rows.sort();
        let mut keys: Vec<String> = rows.into_iter().map(|(_, key)| key).collect();
        sort(&mut keys, reading, showing.sorting);

        keys.into_iter().map(RowKey::of).collect()
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let key = row.key.as_str();
        let mut cells = vec![
            Cell::plain(what(key, item)),
            Cell::plain(program(key, item)),
        ];
        if room.holds(ROOM_FOR_THE_RUNTIME) {
            cells.push(Cell::plain(runtime(key, item)));
        }
        cells.push(Cell::plain(may_take_the_host(key, item)));
        cells.push(Cell::plain(mounted(key, item)));

        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let key = row.key.as_str();
        every_field(
            Kind::of(key).map_or("row", |kind| kind.name()),
            &identity(item),
            key,
            item,
            &means(item),
        )
    }

    fn row_for(&self, reading: &Snapshot, named: &str) -> Option<String> {
        if reading.items.contains_key(named) {
            return Some(named.to_string());
        }

        reading
            .items
            .iter()
            .find(|(key, item)| Kind::of(key) == Some(Kind::Container) && identity(item) == named)
            .map(|(key, _)| key.clone())
    }

    fn holds(&self, reading: &Snapshot, key: &str) -> bool {
        self.row_for(reading, key).is_some()
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(showing)
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(notices::nothing_read())
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn nothing_was_read(&self) -> &'static str {
        "No container and no runtime socket is listed here: nothing was read."
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
    let Some(item) = reading.items.get(key) else {
        return String::new();
    };
    match by {
        1 => what(key, item),
        2 => program(key, item),
        3 => runtime(key, item),
        4 => may_take_the_host(key, item),
        5 => mounted(key, item),
        _ => String::new(),
    }
}
