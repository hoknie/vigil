use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{RowKey, Showing, time_of_day};

use super::fields::{free_step, text};
use crate::types::Family;

pub(super) fn footer(reading: &Snapshot, showing: &Showing<'_>, rows: &[RowKey]) -> String {
    let shown = rows.len();
    let whole = reading.items.len();

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{shown} row(s) in this reading, read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {whole} row(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    let filesystems: Vec<(&str, &Value)> = rows
        .iter()
        .filter(|row| Family::of(&row.key) == Some(Family::Filesystem))
        .filter_map(|row| {
            reading
                .items
                .get(&row.key)
                .map(|item| (row.key.as_str(), item))
        })
        .collect();

    if let Some((step, _, fullest)) = filesystems
        .iter()
        .filter_map(|(key, item)| free_step(item).map(|step| (step, *key, *item)))
        .min_by_key(|(step, key, _)| (*step, *key))
    {
        parts.push(format!(
            "least room on {}, {step}% free",
            text(fullest, "mount")
        ));
    }
    if !filesystems.is_empty() {
        parts.push(format!("{} filesystem(s)", filesystems.len()));
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
