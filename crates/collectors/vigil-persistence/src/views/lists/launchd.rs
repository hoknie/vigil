use serde_json::Value;
use vigil_view::{Column, Width};

use super::super::fields::{flag, text};
use super::units::first_command;

pub(crate) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("JOB", Width::Least(28)),
            Column::new("KIND", Width::Fixed(6)),
            Column::new("WHOSE", Width::Fixed(8)),
            Column::new("RUNS AS", Width::Fixed(10)),
            Column::new("WHAT IT RUNS", Width::Share(3)),
            Column::new("WHEN", Width::Share(2)),
            Column::new("PATH", Width::Share(3)),
        ],
        false => vec![
            Column::new("JOB", Width::Least(28)),
            Column::new("KIND", Width::Fixed(6)),
            Column::new("RUNS AS", Width::Fixed(10)),
            Column::new("WHAT IT RUNS", Width::Share(3)),
        ],
    }
}

pub(crate) fn cells(key: &str, item: &Value, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(item, "name").unwrap_or(key).to_string(),
        text(item, "domain").unwrap_or("?").to_string(),
    ];
    if wide {
        cells.push(whose(item));
    }
    cells.push(text(item, "run_as").unwrap_or("?").to_string());
    cells.push(runs(item));
    if wide {
        cells.push(text(item, "schedule").unwrap_or("—").to_string());
        cells.push(text(item, "path").unwrap_or("—").to_string());
    }
    cells
}

pub(crate) fn whose(item: &Value) -> String {
    match (text(item, "scope"), text(item, "owner")) {
        (Some("person"), Some(owner)) => owner.to_string(),
        (Some("vendor"), _) => "macOS".to_string(),
        (Some("system"), _) => "this Mac".to_string(),
        (_, _) => "?".to_string(),
    }
}

fn runs(item: &Value) -> String {
    if !flag(item, "understood") && flag(item, "readable") {
        return "not a job launchd could read".to_string();
    }
    first_command(item)
}
