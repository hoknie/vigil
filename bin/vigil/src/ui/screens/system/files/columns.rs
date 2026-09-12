use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::{digest, held, mode, owner, standing, what};
use super::row::Row;
use crate::ui::helpers::layout::column;

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec![
            "KIND", "PATH", "MODE", "OWNER", "SIZE", "SHA256", "STANDING",
        ]),
        false => TableRow::new(vec!["KIND", "PATH", "MODE", "OWNER", "STANDING"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(9),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(9),
            Constraint::Length(13),
            Constraint::Length(17),
        ],
        false => vec![
            Constraint::Length(9),
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Length(7),
            Constraint::Length(17),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool, columns: &[usize]) -> TableRow<'static> {
    let mut cells = vec![
        row.kind.name().to_string(),
        what(row),
        mode(row),
        owner(row),
    ];
    if wide {
        cells.push(held(row));
        cells.push(digest(row));
    }
    cells.push(standing(row));

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
