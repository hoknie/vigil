use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::screens::programs::{flag, text};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["WHO", "WHEN", "COMMAND", "FROM THE FILE"]),
        false => TableRow::new(vec!["WHO", "WHEN", "COMMAND"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Fill(4),
            Constraint::Fill(3),
        ],
        false => vec![
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Fill(4),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(row.item, "user").unwrap_or("?").to_string(),
        text(row.item, "schedule").unwrap_or("?").to_string(),
        command(row),
    ];
    if wide {
        cells.push(text(row.item, "source").unwrap_or("—").to_string());
    }
    cells
}

pub(super) fn command(row: &Row<'_>) -> String {
    let command = text(row.item, "command").unwrap_or("?");
    match flag(row.item, "command_redacted") {
        true => format!("{command}  ← part hidden before writing"),
        false => command.to_string(),
    }
}
