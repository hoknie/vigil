use std::collections::BTreeSet;

use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

use super::rows::{nested, rows};
use super::tree::tree;
use crate::types::{Kind, List};

pub(super) fn tally(reading: &Snapshot, list: List, showing: &Showing<'_>) -> String {
    let held = reading
        .items
        .keys()
        .filter(|key| List::holding(key) == list && !Kind::of(key).mark())
        .count();
    let whole = reading.items.len();
    let listed = rows(reading, list, showing);
    let shown = listed
        .iter()
        .filter(|row| !Kind::of(&row.key).mark())
        .count();

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

    if nested(list, showing) && {
        let keys: BTreeSet<&str> = listed.iter().map(|row| row.key.as_str()).collect();
        tree(&reading.items)
            .iter()
            .any(|placed| placed.parents > 1 && keys.contains(placed.key.as_str()))
    } {
        parts.push("+N means N more pull it in".to_string());
    }

    let marked = reading
        .items
        .keys()
        .filter(|key| List::holding(key) == list && Kind::of(key).mark())
        .count();
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
