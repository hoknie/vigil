use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing, time_of_day};

use super::detail;
use super::fields::{marked, text};
use super::launches;

const ROOM_FOR_THE_PATH: u16 = 118;

pub struct Launches;

impl Pane for Launches {
    fn name(&self) -> &'static str {
        "launches"
    }

    fn caption(&self) -> &'static str {
        "LAUNCHES"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED LAUNCH"
    }

    fn about(&self) -> &'static str {
        "one row per person and program the kernel's audit records have seen run; rows are \
         never removed from it"
    }

    fn reads(&self) -> &'static str {
        "launches"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        launches::columns(room.holds(ROOM_FOR_THE_PATH))
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<(bool, String)> = reading
            .items
            .iter()
            .filter(|(key, item)| showing.matches(key, item))
            .map(|(key, _)| (!marked(key), key.clone()))
            .collect();

        rows.sort();
        rows.into_iter().map(|(_, key)| RowKey::of(key)).collect()
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        match reading.items.get(&row.key) {
            Some(item) => launches::cells(&row.key, item, room.holds(ROOM_FOR_THE_PATH))
                .into_iter()
                .map(Cell::plain)
                .collect(),
            None => Vec::new(),
        }
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        if marked(&row.key) {
            return vec![
                Piece::Warning("THE READING ITSELF".to_string()),
                Piece::Blank,
                Piece::warning(
                    text(item, "reason").unwrap_or("part of this reading could not be taken"),
                ),
                Piece::Blank,
                Piece::field("object", row.key.clone()),
            ];
        }

        let mut said = detail::launch(item);
        said.push(Piece::key(row.key.clone()));
        said
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let held = reading.items.keys().filter(|key| !marked(key)).count();
        let mut parts = vec![match showing.holding_back() {
            false => format!(
                "{held} launch(es), read at {}",
                time_of_day(&reading.taken_at)
            ),
            true => format!(
                "{shown} of {held} launch(es), read at {}",
                time_of_day(&reading.taken_at)
            ),
        }];

        let unnamed = reading.items.keys().filter(|key| marked(key)).count();
        if unnamed > 0 {
            parts.push(format!("{unnamed} row(s) about the reading itself"));
        }
        parts.push("this list only grows".to_string());
        if showing.elsewhere > 0 {
            parts.push(format!(
                "{} other list(s) narrowed by a search of their own",
                showing.elsewhere
            ));
        }
        if let Some(reason) = showing.note {
            parts.push(format!("incomplete: {reason}"));
        }

        parts.join(" · ")
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        launches::empty(showing)
    }

    fn nothing_was_read(&self) -> &'static str {
        "No launch is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false)
    }
}
