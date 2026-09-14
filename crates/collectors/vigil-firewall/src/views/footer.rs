use vigil_model::Snapshot;
use vigil_view::{Counts, RowKey, Showing, time_of_day};

use super::fields::{ACCEPT, DROP, hooked_on_input, policy};
use super::rows::summary;
use crate::types::Kind;

const ON_THE_INPUT_HOOK: &str = "on the input hook";

pub(super) fn counts(reading: &Snapshot) -> Counts {
    let hooked = summary(reading)
        .map(|item| format!("{} on the input hook", hooked_on_input(item)))
        .into_iter()
        .collect();
    Counts::default().saying(ON_THE_INPUT_HOOK, hooked)
}

pub(super) fn tallied(
    reading: &Snapshot,
    showing: &Showing<'_>,
    rows: &[RowKey],
    counts: &Counts,
) -> String {
    let shown = rows.len();
    let read_at = time_of_day(&reading.taken_at);

    let mut parts = vec![match showing.holding_back() {
        false => format!("{shown} row(s) in this reading, read at {read_at}"),
        true => format!(
            "{shown} of {} row(s), read at {read_at}",
            reading.items.len()
        ),
    }];
    if let Some(policies) = by_policy(reading, rows) {
        parts.push(policies);
    }
    parts.extend(counts.words(ON_THE_INPUT_HOOK).iter().cloned());
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" · ")
}

fn by_policy(reading: &Snapshot, rows: &[RowKey]) -> Option<String> {
    let mut dropping = 0;
    let mut accepting = 0;

    for row in rows {
        if Kind::of(&row.key) != Some(Kind::Chain) {
            continue;
        }
        let Some(item) = reading.items.get(&row.key) else {
            continue;
        };
        match policy(&row.key, item).as_str() {
            DROP => dropping += 1,
            ACCEPT => accepting += 1,
            _ => {}
        }
    }

    match dropping + accepting {
        0 => None,
        _ => Some(format!(
            "{dropping} chain(s) dropping, {accepting} accepting by default"
        )),
    }
}
