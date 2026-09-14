use vigil_model::{KillTarget, Snapshot};
use vigil_view::{
    Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Sorting,
    Toggle, Width, basename, haystack,
};

use super::detail;
use super::fields::{command, endpoint, pid, program, protocol, user};
use super::footer;
use super::notices;
use super::tally::tally;

pub(super) const ROOM_FOR_THE_COMMAND: u16 = 118;

pub(super) struct Flat;

impl Pane for Flat {
    fn name(&self) -> &str {
        "sockets"
    }

    fn caption(&self) -> &str {
        "LISTENING"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED SOCKET"
    }

    fn about(&self) -> &str {
        "one row per listening socket: the reading itself, and what a finding is keyed by"
    }

    fn reads(&self) -> &str {
        "ports"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("PROTO", Width::Fixed(5)),
            Column::new("ADDRESS", Width::Least(22)),
            Column::new("USER", Width::Fixed(10)),
            Column::new("PID", Width::Fixed(7)),
            Column::new("PROGRAM", Width::Share(1)),
        ];
        if room.holds(ROOM_FOR_THE_COMMAND) {
            columns.push(Column::new("COMMAND", Width::Share(2)));
        }
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<RowKey> = passing(reading, showing)
            .into_iter()
            .map(|(key, _)| RowKey::of(key))
            .collect();
        sort(&mut rows, reading, showing.sorting);
        rows
    }

    fn index(&self, reading: &Snapshot, showing: &Showing<'_>) -> Option<Index> {
        let columns = self.sorted_by().len();
        let mut index = Index::new(columns);
        for (key, item) in wanted(reading, showing) {
            index.push(
                RowKey::of(key.clone()),
                &haystack(key, item),
                0,
                (1..=columns)
                    .map(|by| sorted_on(reading, key, by))
                    .collect(),
            );
        }
        Some(index)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let mut cells = vec![
            Cell::plain(protocol(item, &row.key)),
            Cell::plain(endpoint(item, &row.key)),
            Cell::plain(user(item)),
            Cell::plain(pid(item)),
            Cell::plain(program(item)),
        ];
        if room.holds(ROOM_FOR_THE_COMMAND) {
            cells.push(Cell::plain(command(item)));
        }
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail::socket(&row.key, item),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown, false)
    }

    fn counts(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        Some(footer::counts(reading))
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &[RowKey],
        counts: &Counts,
    ) -> String {
        footer::tallied(reading, showing, rows, counts, false)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(showing)
    }

    fn toggles(&self) -> Vec<Toggle> {
        kinds()
    }

    fn nothing_was_read(&self) -> &'static str {
        "No listening socket is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().killed(KillTarget::Socket)
    }
}

pub(super) fn kinds() -> Vec<Toggle> {
    vec![
        Toggle::new('t', "tcp"),
        Toggle::new('T', "tcp6"),
        Toggle::new('u', "udp"),
        Toggle::new('U', "udp6"),
        Toggle::new('X', "unix"),
    ]
}

pub(super) fn passing<'a>(
    reading: &'a Snapshot,
    showing: &Showing<'_>,
) -> Vec<(&'a String, &'a serde_json::Value)> {
    wanted(reading, showing)
        .filter(|(key, item)| showing.matches(key, item))
        .collect()
}

pub(super) fn wanted<'a>(
    reading: &'a Snapshot,
    showing: &Showing<'_>,
) -> impl Iterator<Item = (&'a String, &'a serde_json::Value)> {
    reading
        .items
        .iter()
        .filter(|(key, item)| showing.wants(protocol(item, key)))
}

fn sort(rows: &mut [RowKey], reading: &Snapshot, sorting: Sorting) {
    if sorting.as_read() {
        return;
    }
    rows.sort_by(|left, right| {
        let ordering = sorted_on(reading, &left.key, sorting.by)
            .cmp(&sorted_on(reading, &right.key, sorting.by));
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
        1 => protocol(item, key).to_string(),
        2 => endpoint(item, key),
        3 => user(item),
        4 => format!(
            "{:>9}",
            crate::types::SocketView::new(item).pid().unwrap_or(0)
        ),
        5 => crate::types::SocketView::new(item)
            .executable()
            .map(basename)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}
