use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

use super::fields::{free_step, text};
use crate::types::Family;

pub(super) fn tally(reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
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

    let filesystems: Vec<&serde_json::Value> = reading
        .items
        .iter()
        .filter(|(key, item)| {
            Family::of(key) == Some(Family::Filesystem) && showing.matches(key, item)
        })
        .map(|(_, item)| item)
        .collect();

    if let Some((step, fullest)) = filesystems
        .iter()
        .filter_map(|item| free_step(item).map(|step| (step, *item)))
        .min_by_key(|(step, _)| *step)
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
