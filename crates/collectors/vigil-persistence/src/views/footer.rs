use vigil_model::Snapshot;
use vigil_view::{Counts, RowKey, Showing, time_of_day};

use super::rows::{nested, parents_of};
use crate::types::{Kind, List};

const HELD: &str = "held";

const ABOUT_THE_READING: &str = "rows about the reading";

pub(super) fn counted(reading: &Snapshot, list: List) -> Counts {
    let (about_the_reading, held) = reading
        .items
        .keys()
        .filter(|key| List::holding(key) == list)
        .fold((0, 0), |(marked, held), key| match Kind::of(key).mark() {
            true => (marked + 1, held),
            false => (marked, held + 1),
        });
    Counts::default()
        .counted(HELD, held)
        .counted(ABOUT_THE_READING, about_the_reading)
}

pub(super) fn footer(
    reading: &Snapshot,
    list: List,
    showing: &Showing<'_>,
    rows: &[RowKey],
    counts: &Counts,
) -> String {
    let held = counts.number(HELD);
    let whole = reading.items.len();
    let shown = rows.iter().filter(|row| !Kind::of(&row.key).mark()).count();

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{held} {} of {whole} in this reading, read at {}",
            list.things(held),
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {held} {}, read at {}",
            list.things(held),
            time_of_day(&reading.taken_at)
        ),
    }];

    if nested(list, showing)
        && rows
            .iter()
            .any(|row| parents_of(&reading.items, &row.key) > 1)
    {
        parts.push("+N means N more pull it in".to_string());
    }

    let marked = counts.number(ABOUT_THE_READING);
    if marked > 0 {
        parts.push(format!("{marked} row(s) about the reading itself"));
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
