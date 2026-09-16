use vigil_model::Snapshot;
use vigil_view::{Counts, Rows, Showing, time_of_day};

use super::fields::{flag, marked};

const HELD: &str = "programs";

const ABOUT_THE_READING: &str = "rows about the reading";

const GONE: &str = "programs whose file is gone";

pub(super) fn counted(reading: &Snapshot) -> Counts {
    let about_the_reading = reading.items.keys().filter(|key| marked(key)).count();
    let gone = reading
        .items
        .iter()
        .filter(|(key, item)| !marked(key) && flag(item, "exe_deleted"))
        .count();
    Counts::default()
        .counted(HELD, reading.items.len() - about_the_reading)
        .counted(ABOUT_THE_READING, about_the_reading)
        .counted(GONE, gone)
}

pub(super) fn footer(
    reading: &Snapshot,
    showing: &Showing<'_>,
    rows: &Rows<'_>,
    counts: &Counts,
) -> String {
    let held = counts.number(HELD);
    let shown = rows.len();
    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{held} program(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {held} program(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    let unnamed = counts.number(ABOUT_THE_READING);
    if unnamed > 0 {
        parts.push(format!("{unnamed} row(s) about the reading itself"));
    }
    let gone = counts.number(GONE);
    if gone > 0 {
        parts.push(format!("{gone} whose file is gone"));
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
