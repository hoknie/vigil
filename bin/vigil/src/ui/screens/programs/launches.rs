use ratatui::layout::Constraint;
use ratatui::widgets::Row as TableRow;

use super::fields::text;
use super::row::Row;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::words::moment;
use crate::ui::screens::ports::basename;
use crate::ui::{Notice, Search, View};

pub(super) fn header(wide: bool) -> TableRow<'static> {
    match wide {
        true => TableRow::new(vec!["WHO", "PROGRAM", "FIRST SEEN", "PATH"]),
        false => TableRow::new(vec!["WHO", "PROGRAM", "FIRST SEEN"]),
    }
}

pub(super) fn widths(wide: bool) -> Vec<Constraint> {
    match wide {
        true => vec![
            Constraint::Length(12),
            Constraint::Min(14),
            Constraint::Fill(2),
            Constraint::Fill(3),
        ],
        false => vec![
            Constraint::Length(12),
            Constraint::Min(14),
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
            who(row),
            basename(text(row.item, "exe").unwrap_or("?")).to_string(),
            text(row.item, "first_seen")
                .map(moment::time_of_day)
                .unwrap_or("?")
                .to_string(),
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

pub(super) fn who(row: &Row<'_>) -> String {
    match text(row.item, "user") {
        Some(user) => user.to_string(),
        None => match row.item.get("auid").and_then(serde_json::Value::as_u64) {
            Some(auid) => format!("login {auid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn empty(view: &View, search: &Search) -> Notice {
    if search.holding_back() {
        return Notice::plain(format!("No launch matches {:?}.", search.query())).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }
    if view.switched_off("launches") {
        return Notice::plain("This agent is not reading what people run here.").saying(
            view.collector_reason("launches")
                .unwrap_or("switched off in the configuration")
                .to_string(),
        );
    }
    match view.collector_note("launches") {
        Some(reason) => Notice::loud("Nothing here, and the records are not being read.")
            .saying(reason.to_string()),
        None => Notice::plain("Nothing has been run since this agent started watching.")
            .saying("This list only grows: a row is never taken off it."),
    }
}
