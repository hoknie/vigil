use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::{flag, strings, text};
use super::row::Row;
use crate::ui::helpers::layout::column;
use crate::ui::screens::ports::basename;
use crate::ui::{Notice, Search, View};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["PROGRAM", "USER", "STARTED BY", "PATH"]),
        false => TableRow::new(vec!["PROGRAM", "USER", "STARTED BY"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Min(14),
            Constraint::Length(12),
            Constraint::Fill(2),
            Constraint::Fill(3),
        ],
        false => vec![
            Constraint::Min(14),
            Constraint::Length(12),
            Constraint::Fill(2),
        ],
    }
}

pub(super) fn cells(row: &Row<'_>, wide: bool, columns: &[usize]) -> TableRow<'static> {
    let mut cells = match row.mark {
        true => vec![
            "—".to_string(),
            "—".to_string(),
            text(row.item, "reason")
                .unwrap_or("this reading is not complete")
                .to_string(),
        ],
        false => vec![
            program(row),
            user(row),
            match strings(row.item, "parents").join(", ") {
                empty if empty.is_empty() => "nothing this agent could see".to_string(),
                parents => parents,
            },
        ],
    };
    if wide {
        cells.push(match row.mark {
            true => "—".to_string(),
            false => text(row.item, "exe").unwrap_or("?").to_string(),
        });
    }

    for (index, cell) in cells.iter_mut().enumerate() {
        if let Some(width) = columns.get(index) {
            *cell = column::fit(cell, *width);
        }
    }
    TableRow::new(cells)
}

pub(super) fn program(row: &Row<'_>) -> String {
    let path = text(row.item, "exe").unwrap_or("?");
    match flag(row.item, "exe_deleted") {
        true => format!("{} (deleted)", basename(path)),
        false => basename(path).to_string(),
    }
}

pub(super) fn user(row: &Row<'_>) -> String {
    match text(row.item, "user") {
        Some(user) => user.to_string(),
        None => match row.item.get("uid").and_then(serde_json::Value::as_u64) {
            Some(uid) => format!("uid {uid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn empty(view: &View, search: &Search) -> Notice {
    if search.holding_back() {
        return Notice::plain(format!("No program matches {:?}.", search.query())).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }
    if view.switched_off("processes") {
        return Notice::plain("This agent is not reading the programs on this host.").saying(
            view.collector_reason("processes")
                .unwrap_or("switched off in the configuration")
                .to_string(),
        );
    }
    match view.collector_note("processes") {
        Some(reason) => Notice::loud("Nothing here, and the reading is incomplete.").saying(
            format!("Reason: {reason}. The summary screen carries the whole of it."),
        ),
        None => Notice::plain("The agent read /proc and found no program it could name.").saying(
            "Every host runs something. Treat this as a reading that saw nothing rather \
             than a host that runs nothing.",
        ),
    }
}
