use vigil_model::Snapshot;
use vigil_view::{Cell, Column, Facet, Notice, Pane, Piece, Room, RowKey, Showing, time_of_day};

use super::detail;
use super::fields::{marked, text};
use super::launches::{self, PROGRAM, USER};

const ROOM_FOR_THE_PATH: u16 = 118;

pub struct Launches;

impl Pane for Launches {
    fn name(&self) -> &str {
        "launches"
    }

    fn caption(&self) -> &str {
        "LAUNCHES"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED LAUNCH"
    }

    fn about(&self) -> &str {
        "one row per person and program the kernel's audit records have seen run, and how \
         many times; rows are never removed from it"
    }

    fn reads(&self) -> &str {
        "launches"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        launches::columns(room.holds(ROOM_FOR_THE_PATH))
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let mut rows: Vec<(&str, &serde_json::Value)> = reading
            .items
            .iter()
            .filter(|(key, item)| showing.matches(key, item))
            .filter(|(key, item)| launches::chosen(key, item, showing))
            .map(|(key, item)| (key.as_str(), item))
            .collect();

        rows.sort_by(|left, right| launches::order(*left, *right, showing.sorting));
        rows.into_iter().map(|(key, _)| RowKey::of(key)).collect()
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

        if let Some(narrowed) = launches::narrowed_to(showing) {
            parts.push(format!("only {narrowed}"));
        }
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

    fn facets(&self, reading: &Snapshot, row: &RowKey) -> Vec<Facet> {
        let Some(item) = reading.items.get(&row.key) else {
            return Vec::new();
        };
        if marked(&row.key) {
            return Vec::new();
        }

        vec![
            Facet::new(USER, launches::who(item)),
            Facet::new(PROGRAM, launches::executable(item)),
        ]
    }
}
