use vigil_model::{KillTarget, Snapshot};
use vigil_view::{
    Assembled, Cell, Column, Counts, Index, Notice, Offers, Pane, Piece, Room, RowKey, Rows,
    Showing, Toggle, Width,
};

use super::detail;
use super::fields::{endpoint, pid, protocol, user};
use super::flat::{ROOM_FOR_THE_COMMAND, kinds};
use super::footer;
use super::gathering::{assembled, indexed};
use super::notices;
use super::programs::{HEADING, Programs, UNRESOLVED, counted, gathered, named_from_the_reading};
use super::tally::tally;

pub(super) struct ByProgram;

impl Pane for ByProgram {
    fn name(&self) -> &str {
        "by program"
    }

    fn caption(&self) -> &str {
        "BY PROGRAM"
    }

    fn detail_caption(&self) -> &'static str {
        "THE SELECTED ROW"
    }

    fn about(&self) -> &str {
        "one row per program, its sockets folded away under it; sockets with no resolved owner \
         are kept apart"
    }

    fn reads(&self) -> &str {
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
        let listed = gathered(reading, showing);
        let names = Programs::of(reading, showing, &listed);
        let (programs, unresolved) = &listed;

        let mut rows = Vec::new();
        for (path, sockets) in programs {
            let heading = format!("{HEADING}{path}");
            let open = showing.opened_up(&heading);
            rows.push(
                RowKey::of(heading.clone())
                    .of_its_own()
                    .gathering(sockets.len())
                    .named(names.name(path))
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
                    rows.push(RowKey::of((*key).clone()).under(1).beneath(UNRESOLVED));
                }
            }
        }
        rows
    }

    fn index(&self, reading: &Snapshot, showing: &Showing<'_>) -> Option<Index> {
        Some(indexed(reading, showing, self.sorted_by().len()))
    }

    fn assemble(
        &self,
        _reading: &Snapshot,
        showing: &Showing<'_>,
        index: &Index,
        ordered: Vec<usize>,
    ) -> Assembled {
        Assembled::Built(assembled(showing, index, &ordered))
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
            return detail::unresolved(sockets(reading, row));
        }
        match row.key.strip_prefix(HEADING) {
            Some(path) => detail::program(path, sockets(reading, row)),
            None => Vec::new(),
        }
    }

    fn tally(&self, reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
        tally(reading, showing, shown, true)
    }

    fn counts(&self, reading: &Snapshot, _showing: &Showing<'_>) -> Option<Counts> {
        Some(footer::counts(reading))
    }

    fn tally_listed(
        &self,
        reading: &Snapshot,
        showing: &Showing<'_>,
        rows: &Rows<'_>,
        counts: &Counts,
    ) -> String {
        footer::tallied(reading, showing, rows, counts, true)
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
        Offers::default().sorted(false).killed(KillTarget::Socket)
    }
}

fn heading(reading: &Snapshot, row: &RowKey, wide: bool) -> Vec<Cell> {
    let count = sockets(reading, row);
    let folded = match row.opened {
        true => "▾ ",
        false => "▸ ",
    };
    let mut cells = match row.key.strip_prefix(HEADING) {
        Some(path) => vec![
            Cell::plain(format!(
                "{folded}{} ({count})",
                row.named
                    .clone()
                    .unwrap_or_else(|| named_from_the_reading(reading, path))
            )),
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

fn sockets(reading: &Snapshot, row: &RowKey) -> usize {
    row.gathers.unwrap_or_else(|| counted(reading, &row.key))
}
