use vigil_model::Snapshot;
use vigil_view::{Arrangement, Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing};

use super::detail;
use super::lists::{cron, files, modules, other, timers, units};
use super::notices;
use super::rows::{TREE, parents_of, rows};
use super::tally::tally;
use crate::types::{Kind, List};

const ROOM_FOR_THE_PATH: u16 = 118;

const LIST: &str = "list";

pub(super) struct Of(pub(super) List);

impl Pane for Of {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn caption(&self) -> &str {
        self.0.caption()
    }

    fn detail_caption(&self) -> &'static str {
        self.0.detail()
    }

    fn about(&self) -> &str {
        self.0.about()
    }

    fn shown(&self, reading: &Snapshot) -> bool {
        self.0.shown(reading)
    }

    fn reads(&self) -> &str {
        "persistence"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let wide = room.holds(ROOM_FOR_THE_PATH);
        match self.0 {
            List::Units => units::columns(wide),
            List::Timers => timers::columns(wide),
            List::Cron => cron::columns(wide),
            List::Modules => modules::columns(wide),
            List::Files => files::columns(wide),
            List::Other => other::columns(wide),
        }
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        rows(reading, self.0, showing)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        let wide = room.holds(ROOM_FOR_THE_PATH);
        let key = row.key.as_str();

        let said = match self.0 {
            List::Units => {
                units::cells(key, item, row.depth, parents_of(&reading.items, key), wide)
            }
            List::Timers => timers::cells(key, item, wide),
            List::Cron => cron::cells(item, wide),
            List::Modules => modules::cells(key, item, Kind::of(key).mark()),
            List::Files => files::cells(key, item, wide),
            List::Other => other::cells(key),
        };

        said.into_iter().map(Cell::plain).collect()
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail::of(&row.key, item, reading),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let _ = shown;
        tally(reading, self.0, showing)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(self.0, showing)
    }

    fn arrangements(&self) -> Vec<Arrangement> {
        match self.0 {
            List::Units => vec![
                Arrangement::new('t', LIST),
                Arrangement::new('t', TREE).saying(
                    "what each unit file says pulls it in — WantedBy, RequiredBy, PartOf, \
                     Wants, Requires — as written in the files, not what systemd has enabled",
                ),
            ],
            _ => Vec::new(),
        }
    }

    fn nothing_was_read(&self) -> &'static str {
        "Nothing this host starts by itself is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false)
    }
}
