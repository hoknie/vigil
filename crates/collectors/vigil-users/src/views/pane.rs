use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing};

use super::cells::cells;
use super::columns::columns;
use super::detail;
use super::notices;
use super::rows::rows;
use super::tally::tally;
use crate::types::Subject;

const ROOM_FOR_WHERE_IT_CAME_FROM: u16 = 118;

pub(super) struct Of(pub(super) Subject);

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
        "users"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        columns(self.0, room.holds(ROOM_FOR_WHERE_IT_CAME_FROM))
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        rows(reading, self.0, showing)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        match reading.items.get(&row.key) {
            Some(item) => cells(
                self.0,
                &row.key,
                item,
                reading,
                room.holds(ROOM_FOR_WHERE_IT_CAME_FROM),
            ),
            None => Vec::new(),
        }
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail::of(&row.key, item, reading),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, self.0, showing, shown)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(self.0, showing)
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(notices::nothing_read())
    }

    fn nothing_was_read(&self) -> &'static str {
        "No account is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false)
    }
}
