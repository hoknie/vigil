use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::words::size;
use crate::ui::screens::programs::{number, strings, text};

const SIZE_COLUMN: usize = 9;

pub(super) fn header(_wide: bool) -> TableRow<'static> {
    TableRow::new(vec![
        "MODULE".to_string(),
        column::right("SIZE", SIZE_COLUMN),
        "USED BY".to_string(),
        "STATE".to_string(),
    ])
}

pub(super) fn widths(_wide: bool) -> Vec<Constraint> {
    vec![
        Constraint::Min(18),
        Constraint::Length(SIZE_COLUMN as u16),
        Constraint::Fill(3),
        Constraint::Length(10),
    ]
}

pub(super) fn cells(row: &Row<'_>, _wide: bool) -> Vec<String> {
    if row.mark() {
        return vec![
            "—".to_string(),
            column::right("—", SIZE_COLUMN),
            text(row.item, "reason")
                .unwrap_or("the loaded modules were not readable")
                .to_string(),
            "unreadable".to_string(),
        ];
    }

    vec![
        text(row.item, "name").unwrap_or(&row.key).to_string(),
        column::right(&held(row), SIZE_COLUMN),
        match strings(row.item, "dependencies").join(", ") {
            empty if empty.is_empty() => "nothing".to_string(),
            used_by => used_by,
        },
        text(row.item, "state").unwrap_or("—").to_string(),
    ]
}

pub(super) fn held(row: &Row<'_>) -> String {
    match number(row.item, "size") {
        Some(size) => size::bytes(size),
        None => "—".to_string(),
    }
}
