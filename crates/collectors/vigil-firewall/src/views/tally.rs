use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

use super::fields::{ACCEPT, DROP, hooked_on_input, policy};
use super::rows::{rows, summary};
use crate::types::Kind;

pub(super) fn tally(reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
    let whole = reading.items.len();

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{shown} row(s) in this reading, read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {whole} row(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    if let Some(policies) = by_policy(reading, showing) {
        parts.push(policies);
    }
    if let Some(item) = summary(reading) {
        parts.push(format!("{} on the input hook", hooked_on_input(item)));
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" · ")
}

fn by_policy(reading: &Snapshot, showing: &Showing<'_>) -> Option<String> {
    let mut dropping = 0;
    let mut accepting = 0;

    for row in rows(reading, showing) {
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
