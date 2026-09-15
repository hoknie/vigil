use serde_json::Value;
use vigil_model::{KillTarget, Snapshot};
use vigil_view::{
    Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows, Showing,
    haystack, time_of_day,
};

use super::detail;
use super::fields::{flag, marked, text};
use super::footer::{counted, footer};
use super::running;

const ROOM_FOR_THE_PATH: u16 = 118;

pub struct Running;

impl Pane for Running {
    fn name(&self) -> &str {
        "running"
    }

    fn caption(&self) -> &str {
        "RUNNING"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED PROGRAM"
    }

    fn about(&self) -> &str {
        "one row per program found running, and as whom"
    }

    fn reads(&self) -> &str {
        "processes"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        running::columns(room.holds(ROOM_FOR_THE_PATH))
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

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        let mut read: Vec<(bool, &String, &Value)> = reading
            .items
            .iter()
            .map(|(key, item)| (!marked(key), key, item))
            .collect();
        read.sort_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));

        let mut index = Index::new(0);
        for (unmarked, key, item) in read {
            index.push(
                RowKey::of(key.as_str()),
                &haystack(key, item),
                u8::from(unmarked),
                Vec::new(),
            );
        }
        Some(index)
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        match reading.items.get(&row.key) {
            Some(item) => running::cells(&row.key, item, room.holds(ROOM_FOR_THE_PATH))
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

        let mut said = detail::running(item);
        said.push(Piece::key(format!("process|{}", row.key)));
        said
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        let held = reading.items.keys().filter(|key| !marked(key)).count();
        let mut parts = vec![match showing.holding_back() {
            false => format!(
                "{held} program(s), read at {}",
                time_of_day(&reading.taken_at)
            ),
            true => format!(
                "{shown} of {held} program(s), read at {}",
                time_of_day(&reading.taken_at)
            ),
        }];

        let unnamed = reading.items.keys().filter(|key| marked(key)).count();
        if unnamed > 0 {
            parts.push(format!("{unnamed} row(s) about the reading itself"));
        }
        let gone = reading
            .items
            .iter()
            .filter(|(key, item)| !marked(key) && flag(item, "exe_deleted"))
            .count();
        if gone > 0 {
            parts.push(format!("{gone} whose file is gone"));
        }
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
        footer(reading, showing, rows, counts)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        running::empty(showing)
    }

    fn nothing_in_the_reading(&self) -> Option<Notice> {
        Some(running::nothing_read())
    }

    fn nothing_was_read(&self) -> &'static str {
        "No program is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false).killed(KillTarget::Program)
    }
}
