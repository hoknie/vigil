use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::row::Row;

pub(super) fn header(_wide: bool) -> TableRow<'static> {
    TableRow::new(vec!["KEY", "WHAT THIS CONSOLE CAN SAY"])
}

pub(super) fn widths(_wide: bool) -> Vec<Constraint> {
    vec![Constraint::Min(24), Constraint::Fill(2)]
}

pub(super) fn cells(row: &Row<'_>, _wide: bool) -> Vec<String> {
    vec![
        row.key.clone(),
        "a kind of object this console does not know".to_string(),
    ]
}
