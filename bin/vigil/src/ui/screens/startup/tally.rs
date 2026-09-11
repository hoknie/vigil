use vigil_model::Snapshot;

use super::rows::{COLLECTOR, in_the_reading, marks, objects, rows};
use super::showing::Showing;
use crate::ui::View;
use crate::ui::helpers::words::moment;

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let list = showing.list;
    let held = objects(view, list);
    let shown = rows(view, showing)
        .into_iter()
        .filter(|row| !row.mark())
        .count();
    let whole = in_the_reading(view);

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{held} {} of {whole} in this reading, read at {}",
            list.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{shown} of {held} {}, read at {}",
            list.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    if let Some(mark) = shared(view, showing) {
        parts.push(mark);
    }
    let marked = marks(view, list);
    if marked > 0 {
        parts.push(format!("{marked} row(s) about the reading itself"));
    }
    if showing.elsewhere > 0 {
        parts.push(format!(
            "{} other list(s) narrowed by a search of their own",
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

fn shared(view: &View, showing: &Showing<'_>) -> Option<String> {
    if !showing.as_a_tree() {
        return None;
    }
    match rows(view, showing)
        .into_iter()
        .filter(|row| row.parents > 1)
        .count()
    {
        0 => None,
        _ => Some("+N means N more pull it in".to_string()),
    }
}
