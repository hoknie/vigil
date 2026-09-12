use vigil_model::Snapshot;

use super::fields::{ACCEPT, DROP, hooked_on_input, policy};
use super::kind::Kind;
use super::rows::{COLLECTOR, in_the_reading, rows, summary};
use super::showing::Showing;
use crate::ui::View;
use crate::ui::helpers::words::moment;

pub(super) fn tally(view: &View, showing: &Showing<'_>, snapshot: &Snapshot, width: u16) -> String {
    let shown = rows(view, showing);
    let whole = in_the_reading(view);

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{} row(s) in this reading, read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{} of {whole} row(s), read at {}",
            shown.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    if let Some(policies) = by_policy(view, showing) {
        parts.push(policies);
    }
    if let Some(item) = summary(view) {
        parts.push(format!("{} on the input hook", hooked_on_input(item)));
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

fn by_policy(view: &View, showing: &Showing<'_>) -> Option<String> {
    let chains = rows(view, showing);
    let chains: Vec<_> = chains
        .iter()
        .filter(|row| row.kind == Kind::Chain)
        .collect();
    if chains.is_empty() {
        return None;
    }

    let dropping = chains.iter().filter(|row| policy(row) == DROP).count();
    let accepting = chains.iter().filter(|row| policy(row) == ACCEPT).count();

    Some(format!(
        "{} base chain(s): {dropping} {DROP}, {accepting} {ACCEPT}",
        chains.len()
    ))
}
