use serde_json::Value;
use vigil_view::{Column, Notice, Showing, Width, basename, time_of_day};

use super::fields::{marked, text};

pub(super) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("WHO", Width::Fixed(12)),
            Column::new("PROGRAM", Width::Least(14)),
            Column::new("FIRST SEEN", Width::Share(2)),
            Column::new("PATH", Width::Share(3)),
        ],
        false => vec![
            Column::new("WHO", Width::Fixed(12)),
            Column::new("PROGRAM", Width::Least(14)),
            Column::new("FIRST SEEN", Width::Share(2)),
        ],
    }
}

pub(super) fn cells(key: &str, item: &Value, wide: bool) -> Vec<String> {
    let mut cells = match marked(key) {
        true => vec![
            "—".to_string(),
            "—".to_string(),
            text(item, "reason")
                .unwrap_or("this reading is not complete")
                .to_string(),
        ],
        false => vec![
            who(item),
            basename(text(item, "exe").unwrap_or("?")).to_string(),
            text(item, "first_seen")
                .map(time_of_day)
                .unwrap_or("?")
                .to_string(),
        ],
    };
    if wide {
        cells.push(match marked(key) {
            true => "—".to_string(),
            false => text(item, "exe").unwrap_or("?").to_string(),
        });
    }

    cells
}

pub(super) fn who(item: &Value) -> String {
    match text(item, "user") {
        Some(user) => user.to_string(),
        None => match item.get("auid").and_then(Value::as_u64) {
            Some(auid) => format!("login {auid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!("No launch matches {:?}.", showing.search)).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match showing.note {
        Some(reason) => Notice::loud("Nothing here, and the records are not being read.")
            .saying(reason.to_string()),
        None => Notice::plain("Nothing has been run since this agent started watching.")
            .saying("This list only grows: a row is never taken off it."),
    }
}
