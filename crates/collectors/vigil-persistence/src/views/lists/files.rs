use serde_json::Value;
use vigil_view::{Column, Width};

use super::super::fields::{flag, number, strings, text};
use crate::types::Kind;

pub(crate) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("FILE", Width::Least(24)),
            Column::new("WHAT IT IS", Width::Fixed(10)),
            Column::new("STATE", Width::Share(2)),
            Column::new("MODE", Width::Fixed(6)),
            Column::new("OWNER", Width::Fixed(10)),
        ],
        false => vec![
            Column::new("FILE", Width::Least(24)),
            Column::new("WHAT IT IS", Width::Fixed(10)),
            Column::new("STATE", Width::Share(2)),
        ],
    }
}

pub(crate) fn cells(key: &str, item: &Value, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(item, "path").unwrap_or(key).to_string(),
        what_it_is(key, item),
        state(key, item),
    ];
    if wide {
        cells.push(text(item, "mode").unwrap_or("—").to_string());
        cells.push(match number(item, "uid") {
            Some(uid) => format!("uid {uid}"),
            None => "—".to_string(),
        });
    }
    cells
}

pub(crate) fn what_it_is(key: &str, item: &Value) -> String {
    match Kind::of(key) {
        Kind::Preload => "preload".to_string(),
        _ => text(item, "family").unwrap_or("script").to_string(),
    }
}

pub(crate) fn state(key: &str, item: &Value) -> String {
    if !flag(item, "present") {
        return "not on this host".to_string();
    }
    if !flag(item, "readable") {
        return "present, not readable".to_string();
    }
    match Kind::of(key) {
        Kind::Preload => match strings(item, "entries").len() {
            0 => "present and empty".to_string(),
            loaded => format!("{loaded} library(s) forced into every process"),
        },
        _ => match text(item, "sha256") {
            Some(digest) => format!("sha256 {}", &digest[..digest.len().min(12)]),
            None => "present".to_string(),
        },
    }
}
