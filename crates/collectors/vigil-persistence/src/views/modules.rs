use serde_json::Value;
use vigil_view::{Column, Width};

use super::fields::{number, right, size_of, strings, text};

const SIZE_COLUMN: usize = 9;

pub(super) fn columns(_wide: bool) -> Vec<Column> {
    vec![
        Column::new("MODULE", Width::Least(18)),
        Column::new("     SIZE", Width::Fixed(SIZE_COLUMN as u16)),
        Column::new("USED BY", Width::Share(3)),
        Column::new("STATE", Width::Fixed(10)),
    ]
}

pub(super) fn cells(key: &str, item: &Value, unreadable: bool) -> Vec<String> {
    if unreadable {
        return vec![
            "—".to_string(),
            right("—", SIZE_COLUMN),
            text(item, "reason")
                .unwrap_or("the loaded modules were not readable")
                .to_string(),
            "unreadable".to_string(),
        ];
    }

    vec![
        text(item, "name").unwrap_or(key).to_string(),
        right(&held(item), SIZE_COLUMN),
        match strings(item, "dependencies").join(", ") {
            empty if empty.is_empty() => "nothing".to_string(),
            used_by => used_by,
        },
        text(item, "state").unwrap_or("—").to_string(),
    ]
}

pub(super) fn held(item: &Value) -> String {
    match number(item, "size") {
        Some(size) => size_of(size),
        None => "—".to_string(),
    }
}
