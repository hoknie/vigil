use serde_json::Value;
use vigil_view::{Column, Width};

use super::super::fields::{flag, text};

pub(crate) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("WHO", Width::Fixed(12)),
            Column::new("WHEN", Width::Fixed(14)),
            Column::new("COMMAND", Width::Share(4)),
            Column::new("FROM THE FILE", Width::Share(3)),
        ],
        false => vec![
            Column::new("WHO", Width::Fixed(12)),
            Column::new("WHEN", Width::Fixed(14)),
            Column::new("COMMAND", Width::Share(4)),
        ],
    }
}

pub(crate) fn cells(item: &Value, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(item, "user").unwrap_or("?").to_string(),
        text(item, "schedule").unwrap_or("?").to_string(),
        command(item),
    ];
    if wide {
        cells.push(text(item, "source").unwrap_or("—").to_string());
    }
    cells
}

pub(crate) fn command(item: &Value) -> String {
    let command = text(item, "command").unwrap_or("?");
    match flag(item, "command_redacted") {
        true => format!("{command}  ← part hidden before writing"),
        false => command.to_string(),
    }
}
