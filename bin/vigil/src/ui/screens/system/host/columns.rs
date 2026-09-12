use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::{device, free, free_inodes, held, how_it_is_mounted, kind_of_store, what};
use super::row::Row;
use crate::ui::helpers::layout::column;

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec![
            "KIND", "WHAT", "DETAIL", "DEVICE", "FREE", "INODES", "SIZE", "MOUNTED",
        ]),
        false => TableRow::new(vec!["KIND", "WHAT", "FREE", "INODES", "SIZE"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(23),
            Constraint::Length(12),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(10),
        ],
        false => vec![
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(9),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool, columns: &[usize]) -> TableRow<'static> {
    let mut cells = vec![row.kind.name().to_string(), what(row)];
    if wide {
        cells.push(kind_of_store(row));
        cells.push(device(row));
    }
    cells.push(free(row));
    cells.push(free_inodes(row));
    cells.push(held(row));
    if wide {
        cells.push(how_it_is_mounted(row));
    }

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
