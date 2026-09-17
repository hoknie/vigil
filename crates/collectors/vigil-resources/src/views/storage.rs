use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{
    Assembled, Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows,
    Showing, Width, bytes, every_field,
};

use super::backing::is_a_filesystem;
use super::backing::{heading_of, taken_apart};
use super::fields::{device, free, free_inodes, means, number, text};
use super::gathering::{assembled, gathered, indexed};
use super::notices;
use super::sharing::{counted, tallied};
use crate::parsers::UNNAMED;
use crate::types::{Backing, Family};

const ROOM_FOR_THE_DEVICE: u16 = 106;

const SORTED_BY: &[&str] = &["WHAT", "FREE", "INODES", "SIZE"];

const OPENED: &str = "\u{25be} ";

const CLOSED: &str = "\u{25b8} ";

pub(super) struct ByStorage;

impl Pane for ByStorage {
    fn name(&self) -> &str {
        "by storage"
    }

    fn caption(&self) -> &str {
        "BY STORAGE"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED ROW"
    }

    fn about(&self) -> &str {
        "one row per disk, pool or device, the filesystems written to it folded away under it, \
         so that two mounts sharing storage are read as one thing"
    }

    fn reads(&self) -> &str {
        "resources"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![Column::new("STORAGE AND MOUNT", Width::Share(1))];
        if room.holds(ROOM_FOR_THE_DEVICE) {
            columns.push(Column::new("TYPE", Width::Fixed(8)));
            columns.push(Column::new("DEVICE", Width::Fixed(16)));
        }
        columns.push(Column::new("FREE", Width::Fixed(6)));
        columns.push(Column::new("INODES", Width::Fixed(6)));
        columns.push(Column::new("SIZE", Width::Fixed(9)));
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows = Vec::new();

        for (heading, under) in gathered(reading, showing) {
            let open = showing.opened_up(&heading);
            rows.push(
                RowKey::of(heading.clone())
                    .of_its_own()
                    .gathering(under.len())
                    .opened(open),
            );
            if !open {
                continue;
            }
            for key in under {
                rows.push(RowKey::of(key.clone()).under(1).beneath(heading.clone()));
            }
        }
        rows
    }

    fn index(&self, reading: &Snapshot, showing: &Showing<'_>) -> Option<Index> {
        Some(indexed(reading, showing, SORTED_BY.len()))
    }

    fn assemble(
        &self,
        _reading: &Snapshot,
        showing: &Showing<'_>,
        index: &Index,
        ordered: Vec<usize>,
    ) -> Assembled {
        Assembled::Gathered(assembled(showing, index, ordered))
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let wide = room.holds(ROOM_FOR_THE_DEVICE);
        let mut cells = match reading.items.get(&row.key) {
            Some(item) => filesystem(item, wide),
            None => heading(reading, row, wide),
        };
        cells.truncate(self.columns(room).len());
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        if let Some(item) = reading.items.get(&row.key) {
            return every_field(
                Family::Filesystem.name(),
                text(item, "mount"),
                &row.key,
                item,
                &means(Family::Filesystem, item),
            );
        }

        store(reading, &row.key)
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tallied(reading, showing, shown, &counted(reading))
    }

    fn counts(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        Some(counted(reading))
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        counts: &Counts,
    ) -> String {
        tallied(reading, showing, rows.len(), counts)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::nothing_on_a_store(showing)
    }

    fn nothing_was_read(&self) -> &'static str {
        "No filesystem is listed here: nothing was read."
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false)
    }
}

fn filesystem(item: &Value, wide: bool) -> Vec<Cell> {
    let mut cells = vec![Cell::plain(format!("  {}", text(item, "mount")))];
    if wide {
        cells.push(Cell::plain(text(item, "type")));
        cells.push(Cell::plain(device(Family::Filesystem, item)));
    }
    cells.push(Cell::plain(free(Family::Filesystem, item)));
    cells.push(Cell::plain(free_inodes(Family::Filesystem, item)));
    cells.push(Cell::plain(bytes(number(item, "total_bytes"))));
    cells
}

fn heading(reading: &Snapshot, row: &RowKey, wide: bool) -> Vec<Cell> {
    let (backing, name) = taken_apart(&row.key).unwrap_or((Backing::Unnamed, UNNAMED));
    let folded = match row.opened {
        true => OPENED,
        false => CLOSED,
    };
    let under = held(reading, &row.key);
    let count = row.gathers.unwrap_or(under.len());

    let mut cells = vec![Cell::plain(format!("{folded}{name} ({count})"))];
    if wide {
        cells.push(Cell::plain(backing.name()));
        cells.push(Cell::plain(""));
    }
    cells.push(Cell::plain(""));
    cells.push(Cell::plain(""));
    cells.push(Cell::plain(bytes(
        under.iter().map(|item| number(item, "total_bytes")).sum(),
    )));
    cells
}

fn store(reading: &Snapshot, key: &str) -> Vec<Piece> {
    let (backing, name) = taken_apart(key).unwrap_or((Backing::Unnamed, UNNAMED));
    let under = held(reading, key);

    let mut pieces = vec![
        Piece::title("storage", name),
        Piece::key(key.to_string()),
        Piece::Blank,
        Piece::text(backing.means()),
        Piece::Blank,
        Piece::heading(format!("{} FILESYSTEM(S) WRITTEN TO IT", under.len())),
    ];
    for item in &under {
        pieces.push(Piece::field(
            text(item, "mount"),
            format!(
                "{}, {} free, {}",
                bytes(number(item, "total_bytes")),
                free(Family::Filesystem, item),
                text(item, "device")
            ),
        ));
    }
    pieces.push(Piece::Blank);
    pieces.push(Piece::field(
        "held here",
        bytes(under.iter().map(|item| number(item, "total_bytes")).sum()),
    ));
    if under.len() > 1 {
        pieces.push(Piece::warning(
            "These filesystems share one store: it filling up, or going away, takes every one \
             of them with it."
                .to_string(),
        ));
    }
    pieces
}

fn held<'a>(reading: &'a Snapshot, key: &str) -> Vec<&'a Value> {
    reading
        .items
        .iter()
        .filter(|(named, item)| is_a_filesystem(named) && heading_of(item) == key)
        .map(|(_, item)| item)
        .collect()
}
