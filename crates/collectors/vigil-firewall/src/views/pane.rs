use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Showing};

use super::columns::{cells, columns};
use super::detail;
use super::footer;
use super::notices;
use super::rows::{SORTED_BY, indexed, rows};
use super::tally::tally;

const ROOM_FOR_THE_TYPE: u16 = 118;

pub(super) struct TheRuleset;

impl Pane for TheRuleset {
    fn name(&self) -> &str {
        "the ruleset"
    }

    fn caption(&self) -> &str {
        "THE RULESET"
    }

    fn about(&self) -> &str {
        "every table, chain and policy the kernel holds, as the ruleset was last written down"
    }

    fn reads(&self) -> &str {
        "firewall"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        columns(room.holds(ROOM_FOR_THE_TYPE))
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        rows(reading, showing)
    }

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        Some(indexed(reading))
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        match reading.items.get(&row.key) {
            Some(item) => cells(&row.key, item, room.holds(ROOM_FOR_THE_TYPE)),
            None => Vec::new(),
        }
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        match reading.items.get(&row.key) {
            Some(item) => detail::of(&row.key, item),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown)
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
        footer::tallied(reading, showing, rows, counts)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(&Snapshot::new("firewall", String::new()), showing)
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(notices::nothing_read())
    }

    fn sorted_by(&self) -> Vec<&'static str> {
        SORTED_BY.to_vec()
    }

    fn nothing_was_read(&self) -> &'static str {
        "No table, chain or policy is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default()
    }
}
