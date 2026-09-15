use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{
    Cell, Column, Counts, Facet, Index, Notice, Pane, Piece, Room, RowKey, Rows, Showing, haystack,
    time_of_day,
};

use super::detail;
use super::fields::{marked, text};
use super::footer::{counted, footer};
use super::launches;
use super::sorted::sorted_on;

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

    fn index(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Index> {
        let mut read: Vec<(u8, &String, &Value)> = reading
            .items
            .iter()
            .map(|(key, item)| (u8::from(!marked(key)), key, item))
            .collect();
        read.sort_by(|left, right| (left.0, left.1).cmp(&(right.0, right.1)));

        let mut index = Index::new(self.sorted_by().len());
        for (group, key, item) in read {
            index.push(
                RowKey::of(key.as_str()),
                &haystack(key, item),
                group,
                sorted_on(item),
            );
            index.faceted(match marked(key) {
                true => Vec::new(),
                false => launches::facets_of(item),
            });
        }
        Some(index)
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

        launches::facets_of(item)
    }
}
