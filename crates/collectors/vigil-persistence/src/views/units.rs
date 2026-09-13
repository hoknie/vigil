use serde_json::Value;
use vigil_view::{Column, Width};

use super::fields::{flag, strings, text};

const DEEPEST_INDENT: usize = 8;

pub(super) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("UNIT", Width::Least(28)),
            Column::new("RUNS AS", Width::Fixed(10)),
            Column::new("WHAT IT RUNS", Width::Share(3)),
            Column::new("PATH", Width::Share(3)),
            Column::new("DESCRIPTION", Width::Share(2)),
        ],
        false => vec![
            Column::new("UNIT", Width::Least(28)),
            Column::new("RUNS AS", Width::Fixed(10)),
            Column::new("WHAT IT RUNS", Width::Share(3)),
        ],
    }
}

pub(super) fn cells(key: &str, item: &Value, depth: u8, parents: usize, wide: bool) -> Vec<String> {
    let mut cells = vec![
        named(key, item, depth, parents),
        text(item, "run_as").unwrap_or("root").to_string(),
        first_command(item),
    ];
    if wide {
        cells.push(text(item, "path").unwrap_or("—").to_string());
        cells.push(text(item, "description").unwrap_or("—").to_string());
    }
    cells
}

pub(super) fn named(key: &str, item: &Value, depth: u8, parents: usize) -> String {
    let name = text(item, "name").unwrap_or(key);
    let indent = " ".repeat((depth as usize * 2).min(DEEPEST_INDENT));
    match parents > 1 {
        true => format!("{indent}{name} +{}", parents - 1),
        false => format!("{indent}{name}"),
    }
}

pub(super) fn first_command(item: &Value) -> String {
    if flag(item, "commands_redacted") {
        return "hidden before it was written down".to_string();
    }
    match strings(item, "commands").first() {
        Some(command) => (*command).to_string(),
        None => match flag(item, "readable") {
            true => "no command in this unit".to_string(),
            false => "this unit file was not readable".to_string(),
        },
    }
}
