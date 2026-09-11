use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::screens::programs::{flag, text};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["TIMER", "WHEN", "ACTIVATES", "PATH"]),
        false => TableRow::new(vec!["TIMER", "WHEN", "ACTIVATES"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Min(18),
            Constraint::Length(16),
            Constraint::Fill(2),
            Constraint::Fill(3),
        ],
        false => vec![
            Constraint::Min(18),
            Constraint::Length(16),
            Constraint::Fill(2),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(row.item, "name").unwrap_or(&row.key).to_string(),
        when(row),
        text(row.item, "activates").unwrap_or("—").to_string(),
    ];
    if wide {
        cells.push(text(row.item, "path").unwrap_or("—").to_string());
    }
    cells
}

pub(super) fn when(row: &Row<'_>) -> String {
    match text(row.item, "on_calendar") {
        Some(calendar) => calendar.to_string(),
        None => match flag(row.item, "on_boot") {
            true => "on boot".to_string(),
            false => "not stated in the file".to_string(),
        },
    }
}
