use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::screens::programs::{flag, strings, text};

const DEEPEST_INDENT: usize = 8;

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec![
            "UNIT",
            "RUNS AS",
            "WHAT IT RUNS",
            "PATH",
            "DESCRIPTION",
        ]),
        false => TableRow::new(vec!["UNIT", "RUNS AS", "WHAT IT RUNS"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Min(28),
            Constraint::Length(10),
            Constraint::Fill(3),
            Constraint::Fill(3),
            Constraint::Fill(2),
        ],
        false => vec![
            Constraint::Min(28),
            Constraint::Length(10),
            Constraint::Fill(3),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool) -> Vec<String> {
    let mut cells = vec![
        named(row),
        text(row.item, "run_as").unwrap_or("root").to_string(),
        first_command(row),
    ];
    if wide {
        cells.push(text(row.item, "path").unwrap_or("—").to_string());
        cells.push(text(row.item, "description").unwrap_or("—").to_string());
    }
    cells
}

pub(super) fn named(row: &Row<'_>) -> String {
    let name = text(row.item, "name").unwrap_or(&row.key);
    let indent = " ".repeat((row.depth * 2).min(DEEPEST_INDENT));
    match row.parents > 1 {
        true => format!("{indent}{name} +{}", row.parents - 1),
        false => format!("{indent}{name}"),
    }
}

pub(super) fn first_command(row: &Row<'_>) -> String {
    if flag(row.item, "commands_redacted") {
        return "hidden before it was written down".to_string();
    }
    match strings(row.item, "commands").first() {
        Some(command) => (*command).to_string(),
        None => match flag(row.item, "readable") {
            true => "no command in this unit".to_string(),
            false => "this unit file was not readable".to_string(),
        },
    }
}
