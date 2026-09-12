use vigil_model::Snapshot;

use crate::types::{Cell, Column, Notice, Offers, Piece, Room, RowKey, Showing, Toggle};

pub trait Pane: Send + Sync {
    fn name(&self) -> &'static str;

    fn about(&self) -> &'static str;

    fn reads(&self) -> &'static str;

    fn columns(&self, room: Room) -> Vec<Column>;

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey>;

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell>;

    fn detail(&self, reading: &Snapshot, row: &RowKey, width: usize) -> Vec<Piece>;

    fn tally(&self, reading: &Snapshot, shown: usize) -> String;

    fn empty(&self, showing: &Showing<'_>) -> Notice;

    fn toggles(&self) -> Vec<Toggle> {
        Vec::new()
    }

    fn offers(&self) -> Offers {
        Offers::default()
    }
}
