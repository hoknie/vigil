use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::basename;
use vigil_view::{Cell, Column, Notice, Offers, Pane, Piece, Room, RowKey, Showing, Toggle, Width};

use super::detail;
use super::fields::{endpoint, holder, pid, protocol, user};
use super::flat::{ROOM_FOR_THE_COMMAND, kinds, passing};
use super::notices;
use super::tally::tally;

const UNRESOLVED: &str = "unresolved";

const HEADING: &str = "program|";

pub(super) struct ByProgram;

impl Pane for ByProgram {
    fn name(&self) -> &'static str {
        "by program"
    }

    fn caption(&self) -> &'static str {
        "BY PROGRAM"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED ROW"
    }

    fn about(&self) -> &'static str {
        "one row per program, its sockets folded away under it; sockets with no resolved owner \
         are kept apart"
    }

    fn reads(&self) -> &'static str {
        "ports"
    }

    fn columns(&self, room: Room) -> Vec<Column> {
        let mut columns = vec![
            Column::new("PROGRAM", Width::Least(20)),
            Column::new("ADDRESS", Width::Least(22)),
            Column::new("USER", Width::Fixed(10)),
            Column::new("PID", Width::Fixed(7)),
        ];
        if room.holds(ROOM_FOR_THE_COMMAND) {
            columns.push(Column::new("WHERE IT IS", Width::Share(1)));
        }
        columns
    }

    fn rows(&self, reading: &Snapshot, showing: &Showing<'_>) -> Vec<RowKey> {
        let (programs, unresolved) = gathered(reading, showing);

        let mut rows = Vec::new();
        for (path, sockets) in &programs {
            let heading = format!("{HEADING}{path}");
            let open = showing.opened_up(&heading);
            rows.push(
                RowKey::of(heading.clone())
                    .of_its_own()
                    .gathering(sockets.len())
                    .opened(open),
            );
            if !open {
                continue;
            }
            for key in sockets {
                rows.push(RowKey::of((*key).clone()).under(1).beneath(heading.clone()));
            }
        }
        if !unresolved.is_empty() {
            rows.push(
                RowKey::of(UNRESOLVED)
                    .of_its_own()
                    .gathering(unresolved.len())
                    .opened(showing.opened_up(UNRESOLVED)),
            );
            if showing.opened_up(UNRESOLVED) {
                for key in unresolved {
                    rows.push(RowKey::of(key.clone()).under(1).beneath(UNRESOLVED));
                }
            }
        }
        rows
    }

    fn cells(&self, reading: &Snapshot, row: &RowKey, room: Room) -> Vec<Cell> {
        let wide = room.holds(ROOM_FOR_THE_COMMAND);
        let mut cells = match reading.items.get(&row.key) {
            Some(item) => {
                let mut cells = vec![
                    Cell::plain(format!("  {}", protocol(item, &row.key))),
                    Cell::plain(endpoint(item, &row.key)),
                    Cell::plain(user(item)),
                    Cell::plain(pid(item)),
                ];
                if wide {
                    cells.push(Cell::plain(""));
                }
                cells
            }
            None => heading(reading, row, wide),
        };
        cells.truncate(self.columns(room).len());
        cells
    }

    fn detail(&self, reading: &Snapshot, row: &RowKey, _width: usize) -> Vec<Piece> {
        if let Some(item) = reading.items.get(&row.key) {
            return detail::socket(&row.key, item);
        }
        if row.key == UNRESOLVED {
            return detail::unresolved(counted(reading, row));
        }
        match row.key.strip_prefix(HEADING) {
            Some(path) => detail::program(path, counted(reading, row)),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown, true)
    }

    fn empty(&self, showing: &Showing<'_>) -> Notice {
        notices::empty(showing)
    }

    fn toggles(&self) -> Vec<Toggle> {
        kinds()
    }

    fn nothing_was_read(&self) -> &'static str {
        "No listening socket is listed here: nothing was read."
    }

    fn offers(&self) -> Offers {
        Offers::default().sorted(false).marked(true)
    }
}

type Gathered<'a> = (Vec<(String, Vec<&'a String>)>, Vec<&'a String>);

fn gathered<'a>(reading: &'a Snapshot, showing: &Showing<'_>) -> Gathered<'a> {
    let mut programs: Vec<(String, Vec<&'a String>)> = Vec::new();
    let mut unresolved: Vec<&'a String> = Vec::new();

    for (key, item) in passing(reading, showing) {
        match holder(item) {
            None => unresolved.push(key),
            Some(path) => match programs.iter_mut().find(|(known, _)| known == path) {
                Some((_, sockets)) => sockets.push(key),
                None => programs.push((path.to_string(), vec![key])),
            },
        }
    }
    programs.sort_by(|left, right| left.0.cmp(&right.0));

    (programs, unresolved)
}

fn heading(reading: &Snapshot, row: &RowKey, wide: bool) -> Vec<Cell> {
    let count = row.gathers.unwrap_or_else(|| counted(reading, row));
    let folded = match row.opened {
        true => "▾ ",
        false => "▸ ",
    };
    let mut cells = match row.key.strip_prefix(HEADING) {
        Some(path) => vec![
            Cell::plain(match ambiguous(reading, path) {
                true => format!("{folded}{path} ({count})"),
                false => format!("{folded}{} ({count})", basename(path)),
            }),
            Cell::plain(""),
            Cell::plain(""),
            Cell::plain(""),
        ],
        None => vec![
            Cell::plain(format!("{folded}owner not resolved ({count})")),
            Cell::plain("permission denied"),
            Cell::plain(""),
            Cell::plain(""),
        ],
    };
    if wide {
        cells.push(Cell::plain(match row.key.strip_prefix(HEADING) {
            Some(path) => path.to_string(),
            None => String::new(),
        }));
    }
    cells
}

fn counted(reading: &Snapshot, row: &RowKey) -> usize {
    match row.key.strip_prefix(HEADING) {
        Some(path) => reading
            .items
            .values()
            .filter(|item| holder(item) == Some(path))
            .count(),
        None => reading
            .items
            .values()
            .filter(|item| is_a_socket(item) && holder(item).is_none())
            .count(),
    }
}

fn ambiguous(reading: &Snapshot, path: &str) -> bool {
    let name = basename(path);
    let mut seen: Vec<&str> = reading
        .items
        .values()
        .filter_map(holder)
        .filter(|other| basename(other) == name)
        .collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len() > 1
}

fn is_a_socket(item: &Value) -> bool {
    crate::types::SocketView::new(item).is_socket()
}
