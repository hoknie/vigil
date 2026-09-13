use serde_json::Value;
use vigil_view::{Column, Width};

use super::fields::{strings, text};

pub(super) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("TIMER", Width::Fixed(18)),
            Column::new("WHEN", Width::Share(5)),
            Column::new("ACTIVATES", Width::Share(3)),
            Column::new("PATH", Width::Share(4)),
        ],
        false => vec![
            Column::new("TIMER", Width::Fixed(16)),
            Column::new("WHEN", Width::Share(5)),
            Column::new("ACTIVATES", Width::Share(3)),
        ],
    }
}

pub(super) fn cells(key: &str, item: &Value, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(item, "name").unwrap_or(key).to_string(),
        when(item),
        text(item, "activates").unwrap_or("—").to_string(),
    ];
    if wide {
        cells.push(text(item, "path").unwrap_or("—").to_string());
    }
    cells
}

pub(super) fn when(item: &Value) -> String {
    let calendar = schedules(item);
    match calendar.len() {
        0 => match text(item, "on_boot") {
            Some(_) => "on boot".to_string(),
            None => "not stated in the file".to_string(),
        },
        1 => calendar[0].to_string(),
        several => format!("{} and {} more", calendar[0], several - 1),
    }
}

pub fn schedules(item: &Value) -> Vec<&str> {
    match text(item, "on_calendar") {
        Some(one) => vec![one],
        None => strings(item, "on_calendar"),
    }
}
