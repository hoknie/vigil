use vigil_model::Snapshot;

use super::rows::{COLLECTOR, in_the_reading, objects, rows};
use super::showing::Showing;
use crate::ui::helpers::words::moment;
use crate::ui::{Search, View};

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let list = showing.list;
    let held = objects(view, list);
    let shown = rows(view, list, showing.search)
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

    let marks = rows(view, list, &Search::default())
        .into_iter()
        .filter(|row| row.mark())
        .count();
    if marks > 0 {
        parts.push(format!("{marks} row(s) about the reading itself"));
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
