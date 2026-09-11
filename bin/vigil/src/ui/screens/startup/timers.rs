use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::screens::programs::{strings, text};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["TIMER", "WHEN", "ACTIVATES", "PATH"]),
        false => TableRow::new(vec!["TIMER", "WHEN", "ACTIVATES"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(18),
            Constraint::Fill(5),
            Constraint::Fill(3),
            Constraint::Fill(4),
        ],
        false => vec![
            Constraint::Length(16),
            Constraint::Fill(5),
            Constraint::Fill(3),
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
    let calendar = schedules(row);
    match calendar.len() {
        0 => match text(row.item, "on_boot") {
            Some(_) => "on boot".to_string(),
            None => "not stated in the file".to_string(),
        },
        1 => calendar[0].to_string(),
        several => format!("{} and {} more", calendar[0], several - 1),
    }
}

pub fn schedules<'a>(row: &Row<'a>) -> Vec<&'a str> {
    match text(row.item, "on_calendar") {
        Some(one) => vec![one],
        None => strings(row.item, "on_calendar"),
    }
}
