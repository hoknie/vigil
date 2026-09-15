use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Rows, Showing, time_of_day};

use super::fields::{marks, present};
use crate::types::Family;

pub(super) fn footer(reading: &Snapshot, showing: &Showing<'_>, rows: &Rows<'_>) -> String {
    let shown = rows.len();
    let whole = reading.items.len();

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{shown} watched path(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {whole} watched path(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    let listed: Vec<(Family, &Value)> = rows
        .iter()
        .filter_map(|row| Some((Family::of(&row.key)?, reading.items.get(&row.key)?)))
        .collect();

    let files = listed
        .iter()
        .filter(|(family, _)| *family == Family::File)
        .count();
    let directories = listed
        .iter()
        .filter(|(family, _)| *family == Family::Directory)
        .count();
    parts.push(format!("{files} file(s), {directories} directory(s)"));

    let gone = listed.iter().filter(|(_, item)| !present(item)).count();
    if gone > 0 {
        parts.push(format!("{gone} not on this host"));
    }
    let marked = listed
        .iter()
        .filter(|(_, item)| !marks(item).is_empty())
        .count();
    if marked > 0 {
        parts.push(format!(
            "{marked} with suid, sgid or a write bit for anyone"
        ));
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
            showing.elsewhere
        ));
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" · ")
}
