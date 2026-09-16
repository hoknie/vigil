use vigil_model::Snapshot;
use vigil_view::{Counts, Rows, Showing, time_of_day};

use super::fields::marked;
use super::launches;

const HELD: &str = "launches";

const ABOUT_THE_READING: &str = "rows about the reading";

pub(super) fn counted(reading: &Snapshot) -> Counts {
    let about_the_reading = reading.items.keys().filter(|key| marked(key)).count();
    Counts::default()
        .counted(HELD, reading.items.len() - about_the_reading)
        .counted(ABOUT_THE_READING, about_the_reading)
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
            "{held} launch(es), read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {held} launch(es), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    if let Some(narrowed) = launches::narrowed_to(showing) {
        parts.push(format!("only {narrowed}"));
    }
    let unnamed = counts.number(ABOUT_THE_READING);
    if unnamed > 0 {
        parts.push(format!("{unnamed} row(s) about the reading itself"));
    }
    parts.push("this list only grows".to_string());
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
