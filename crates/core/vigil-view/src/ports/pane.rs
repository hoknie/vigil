use vigil_model::{AccountChange, Changing, Snapshot};

use crate::types::{
    Arrangement, Assembled, Cell, Column, Counts, Facet, Form, Index, Notice, Offers, Piece, Room,
    RowKey, Rows, Showing, Toggle,
};

const NARROW: u16 = 80;

const READ_NOT_CHANGED: &str = "this list is read, not changed";

pub trait Pane: Send + Sync {
    fn name(&self) -> &str;

    fn caption(&self) -> &str;

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED ROW"
    }

    fn about(&self) -> &str;

    fn shown(&self, _reading: &Snapshot) -> bool {
        true
    }

    fn reads(&self) -> &str;

    fn columns(&self, room: Room) -> Vec<Column>;

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey>;

    fn index(&self, _reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        None
    }

    fn assemble(
        &self,
        _reading: &Snapshot,
        _showing: &Showing<'_>,
        _index: &Index,
        ordered: Vec<usize>,
    ) -> Assembled {
        Assembled::Ordered(ordered)
    }

    fn counts(&self, _reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        None
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        _counts: &Counts,
    ) -> String {
        self.tally(reading, showing, rows.len())
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell>;

    fn detail(&self, reading: &Snapshot, row: &RowKey, width: usize) -> Vec<Piece>;

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String;

    fn empty(&self, showing: &Showing<'_>) -> Notice;

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        None
    }

    fn nothing_was_read(&self) -> &'static str {
        "Nothing is listed here: nothing was read."
    }

    fn row_for(&self, reading: &Snapshot, named: &str) -> Option<String> {
        match reading.items.contains_key(named) {
            true => Some(named.to_string()),
            false => None,
        }
    }

    fn holds(&self, reading: &Snapshot, key: &str) -> bool {
        self.rows(reading, &Showing::default())
            .iter()
            .any(|row| row.key == key)
    }

    fn toggles(&self) -> Vec<Toggle> {
        Vec::new()
    }

    fn arrangements(&self) -> Vec<Arrangement> {
        Vec::new()
    }

    fn facets(&self, _reading: &Snapshot, _row: &RowKey) -> Vec<Facet> {
        Vec::new()
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        self.columns(Room::of(NARROW))
            .into_iter()
            .map(|column| column.header)
            .collect()
    }

    fn offers(&self) -> Offers {
        Offers::default()
    }

    fn history(&self, _reading: &Snapshot, _row: &RowKey) -> Vec<Piece> {
        Vec::new()
    }

    fn form(
        &self,
        _reading: &Snapshot,
        _row: Option<&RowKey>,
        _changing: Changing,
    ) -> Result<Form, String> {
        Err(READ_NOT_CHANGED.to_string())
    }

    fn change(
        &self,
        _reading: &Snapshot,
        _row: Option<&RowKey>,
        _changing: Changing,
        _form: Option<&Form>,
    ) -> Result<AccountChange, String> {
        Err(READ_NOT_CHANGED.to_string())
    }
}
