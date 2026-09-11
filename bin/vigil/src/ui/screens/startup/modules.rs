use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;
use crate::ui::screens::programs::{number, strings, text};

pub(super) fn header(_wide: bool) -> TableRow<'static> {
    TableRow::new(vec!["MODULE", "SIZE", "USED BY", "STATE"])
}

pub(super) fn widths(_wide: bool) -> Vec<Constraint> {
    vec![
        Constraint::Min(18),
        Constraint::Length(10),
        Constraint::Fill(3),
        Constraint::Length(10),
    ]
}

pub(super) fn cells(row: &Row<'_>, _wide: bool) -> Vec<String> {
    if row.mark() {
        return vec![
            "—".to_string(),
            "—".to_string(),
            text(row.item, "reason")
                .unwrap_or("the loaded modules were not readable")
                .to_string(),
            "unreadable".to_string(),
        ];
    }

    vec![
        text(row.item, "name").unwrap_or(&row.key).to_string(),
        match number(row.item, "size") {
            Some(size) => format!("{size} B"),
            None => "—".to_string(),
        },
        match strings(row.item, "dependencies").join(", ") {
            empty if empty.is_empty() => "nothing".to_string(),
            used_by => used_by,
        },
        text(row.item, "state").unwrap_or("—").to_string(),
    ]
}
