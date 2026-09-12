use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::{may_take_the_host, mounted, program, runtime, what};
use super::row::Row;
use crate::ui::helpers::layout::column;

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec![
            "CONTAINER",
            "PROGRAM",
            "RUNTIME",
            "SYS_ADMIN",
            "HOST PATHS",
        ]),
        false => TableRow::new(vec!["CONTAINER", "PROGRAM", "SYS_ADMIN", "PATHS"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(14),
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Length(9),
            Constraint::Length(10),
        ],
        false => vec![
            Constraint::Length(14),
            Constraint::Fill(1),
            Constraint::Length(9),
            Constraint::Length(5),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool, columns: &[usize]) -> TableRow<'static> {
    let mut cells = vec![what(row), program(row)];
    if wide {
        cells.push(runtime(row));
    }
    cells.push(may_take_the_host(row));
    cells.push(mounted(row));

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
