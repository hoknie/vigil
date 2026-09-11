use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::kind::Kind;
use super::row::Row;
use crate::ui::screens::programs::{flag, number, strings, text};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["FILE", "WHAT IT IS", "STATE", "MODE", "OWNER"]),
        false => TableRow::new(vec!["FILE", "WHAT IT IS", "STATE"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Fill(2),
            Constraint::Length(6),
            Constraint::Length(10),
        ],
        false => vec![
            Constraint::Min(24),
            Constraint::Length(10),
            Constraint::Fill(2),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool) -> Vec<String> {
    let mut cells = vec![
        text(row.item, "path").unwrap_or(&row.key).to_string(),
        what_it_is(row),
        state(row),
    ];
    if wide {
        cells.push(text(row.item, "mode").unwrap_or("—").to_string());
        cells.push(match number(row.item, "uid") {
            Some(uid) => format!("uid {uid}"),
            None => "—".to_string(),
        });
    }
    cells
}

pub(super) fn what_it_is(row: &Row<'_>) -> String {
    match row.kind {
        Kind::Preload => "preload".to_string(),
        _ => text(row.item, "family").unwrap_or("script").to_string(),
    }
}

pub(super) fn state(row: &Row<'_>) -> String {
    if !flag(row.item, "present") {
        return "not on this host".to_string();
    }
    if !flag(row.item, "readable") {
        return "present, not readable".to_string();
    }
    match row.kind {
        Kind::Preload => match strings(row.item, "entries").len() {
            0 => "present and empty".to_string(),
            loaded => format!("{loaded} library(s) forced into every process"),
        },
        _ => match text(row.item, "sha256") {
            Some(digest) => format!("sha256 {}", &digest[..digest.len().min(12)]),
            None => "present".to_string(),
        },
    }
}
