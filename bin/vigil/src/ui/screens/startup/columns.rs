use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use super::{cron, files, modules, other, timers, units};
use crate::ui::Startup;
use crate::ui::helpers::layout::column;

pub(super) fn header(list: Startup, wide: bool) -> TableRow<'static> {
    match list {
        Startup::Units => units::header(wide),
        Startup::Timers => timers::header(wide),
        Startup::Cron => cron::header(wide),
        Startup::Modules => modules::header(wide),
        Startup::Files => files::header(wide),
        Startup::Other => other::header(wide),
    }
}

pub(super) fn widths(list: Startup, wide: bool) -> Vec<Constraint> {
    match list {
        Startup::Units => units::widths(wide),
        Startup::Timers => timers::widths(wide),
        Startup::Cron => cron::widths(wide),
        Startup::Modules => modules::widths(wide),
        Startup::Files => files::widths(wide),
        Startup::Other => other::widths(wide),
    }
}

pub(super) fn cells(
    list: Startup,
    row: &Row<'_>,
    wide: bool,
    columns: &[usize],
) -> TableRow<'static> {
    let mut cells = match list {
        Startup::Units => units::cells(row, wide),
        Startup::Timers => timers::cells(row, wide),
        Startup::Cron => cron::cells(row, wide),
        Startup::Modules => modules::cells(row, wide),
        Startup::Files => files::cells(row, wide),
        Startup::Other => other::cells(row, wide),
    };

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}
