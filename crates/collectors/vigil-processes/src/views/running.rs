use serde_json::Value;
use vigil_view::{Column, Notice, Showing, Width, basename};

use super::fields::{flag, marked, strings, text};

pub(super) fn columns(wide: bool) -> Vec<Column> {
    match wide {
        true => vec![
            Column::new("PROGRAM", Width::Least(14)),
            Column::new("USER", Width::Fixed(12)),
            Column::new("STARTED BY", Width::Share(2)),
            Column::new("PATH", Width::Share(3)),
        ],
        false => vec![
            Column::new("PROGRAM", Width::Least(14)),
            Column::new("USER", Width::Fixed(12)),
            Column::new("STARTED BY", Width::Share(2)),
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
            program(item),
            user(item),
            match strings(item, "parents").join(", ") {
                empty if empty.is_empty() => "nothing this agent could see".to_string(),
                parents => parents,
            },
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

pub(super) fn program(item: &Value) -> String {
    let path = text(item, "exe").unwrap_or("?");
    match flag(item, "exe_deleted") {
        true => format!("{} (deleted)", basename(path)),
        false => basename(path).to_string(),
    }
}

pub(super) fn user(item: &Value) -> String {
    match text(item, "user") {
        Some(user) => user.to_string(),
        None => match item.get("uid").and_then(serde_json::Value::as_u64) {
            Some(uid) => format!("uid {uid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    if showing.holding_back() {
        return Notice::plain(format!("No program matches {:?}.", showing.search)).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match showing.note {
        Some(reason) => Notice::loud("Nothing here, and the reading is incomplete.").saying(
            format!("Reason: {reason}. The summary screen carries the whole of it."),
        ),
        None => Notice::plain("The agent read /proc and found no program it could name.").saying(
            "Every host runs something. Treat this as a reading that saw nothing rather than \
             a host that runs nothing.",
        ),
    }
}

pub(super) fn nothing_read() -> Notice {
    Notice::loud("The reading holds no program at all.").saying(
        "Every host runs something, so a reading with no row in it is a failed reading rather \
         than a quiet host.",
    )
}
