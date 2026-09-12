use vigil_model::Snapshot;

use super::fields::{SYS_ADMIN, capabilities, mounts_readable, truncated};
use super::kind::Kind;
use super::rows::{COLLECTOR, in_the_reading, rows, sockets};
use super::showing::Showing;
use crate::ui::View;
use crate::ui::helpers::words::moment;

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let shown = rows(view, showing);
    let whole = in_the_reading(view);

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{} container(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{} of {whole} container(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    parts.push(match sockets(view).as_slice() {
        [] => "no runtime socket on this host".to_string(),
        held => held
            .iter()
            .map(|(path, mode)| format!("socket {path} {mode}"))
            .collect::<Vec<String>>()
            .join(", "),
    });
    let privileged = shown
        .iter()
        .filter(|row| capabilities(row.item).is_some_and(|held| held & SYS_ADMIN != 0))
        .count();
    if privileged > 0 {
        parts.push(format!("{privileged} holding SYS_ADMIN"));
    }
    let unreadable = shown
        .iter()
        .filter(|row| row.kind == Kind::Container)
        .filter(|row| capabilities(row.item).is_none() || !mounts_readable(row.item))
        .count();
    if unreadable > 0 {
        parts.push(format!("{unreadable} could not be read whole"));
    }
    if shown.iter().any(|row| truncated(row.item)) {
        parts.push("one holds more host paths than this reading carries".to_string());
    }
    if !showing.sorting.as_read() {
        parts.push(format!(
            "sorted by {}",
            showing.sorting.describe(super::SORTED_BY)
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
