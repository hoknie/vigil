use vigil_model::Snapshot;

use super::super::showing::Showing;
use super::fields::{free_step, text};
use super::kind::Kind;
use super::rows::{COLLECTOR, in_the_reading, rows};
use crate::ui::View;
use crate::ui::helpers::words::moment;

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let shown = rows(view, showing);
    let whole = in_the_reading(view);

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{} row(s) in this reading, read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{} of {whole} row(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    let filesystems: Vec<&super::row::Row<'_>> = shown
        .iter()
        .filter(|row| row.kind == Kind::Filesystem)
        .collect();
    if let Some(fullest) = filesystems
        .iter()
        .filter_map(|row| free_step(row.item).map(|step| (step, row)))
        .min_by_key(|(step, _)| *step)
    {
        parts.push(format!(
            "least room on {}, {}% free",
            text(fullest.1.item, "mount"),
            fullest.0
        ));
    }
    if !filesystems.is_empty() {
        parts.push(format!("{} filesystem(s)", filesystems.len()));
    }
    if !showing.sorting.as_read() {
        parts.push(format!(
            "sorted by {}",
            showing.sorting.describe(super::SORTED_BY)
        ));
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search",
            showing.elsewhere
        ));
    }
    if let Some(reason) = view.collector_note(COLLECTOR) {
        parts.push(format!("incomplete: {reason}"));
    }

    let room = (width as usize).saturating_sub(1);
    let mut line = String::new();
    for part in parts {
        let next = match line.is_empty() {
            true => part,
            false => format!("{line} · {part}"),
        };
        if next.chars().count() > room {
            break;
        }
        line = next;
    }
    format!(" {line}")
}
