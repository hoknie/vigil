use vigil_model::Snapshot;

use super::super::showing::Showing;
use super::fields::{marks, present};
use super::kind::Kind;
use super::rows::{COLLECTOR, in_the_reading, rows};
use crate::ui::View;
use crate::ui::helpers::words::moment;

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let shown = rows(view, showing);
    let whole = in_the_reading(view);

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{} watched path(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{} of {whole} watched path(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    let files = shown.iter().filter(|row| row.kind == Kind::File).count();
    let directories = shown
        .iter()
        .filter(|row| row.kind == Kind::Directory)
        .count();
    parts.push(format!("{files} file(s), {directories} directory(s)"));

    let gone = shown.iter().filter(|row| !present(row.item)).count();
    if gone > 0 {
        parts.push(format!("{gone} not on this host"));
    }
    let marked = shown.iter().filter(|row| !marks(row).is_empty()).count();
    if marked > 0 {
        parts.push(format!(
            "{marked} with suid, sgid or a write bit for anyone"
        ));
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
