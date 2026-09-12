use serde_json::Value;
use vigil_model::Snapshot;

use super::arrangement::Arrangement;
use super::showing::Showing;
use crate::ui::helpers::words::moment;

pub(super) fn tally(
    snapshot: &Snapshot,
    showing: &Showing<'_>,
    shown: usize,
    width: u16,
) -> String {
    let protocols = &showing.protocols;
    let arrangement = showing.arrangement;
    let unresolved = snapshot
        .items
        .values()
        .filter(|item| item.get("owner_resolved").and_then(Value::as_bool) == Some(false))
        .count();
    let narrowed = protocols.holding_back() || showing.search.holding_back();

    let mut parts = vec![match narrowed {
        false => format!(
            "{} listening socket(s), read at {}",
            snapshot.items.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{} of {} listening socket(s), read at {}",
            shown,
            snapshot.items.len(),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];
    if unresolved > 0 {
        parts.push(format!("{unresolved} whose process is not visible"));
    }
    if protocols.holding_back() {
        parts.push(protocols.describe());
    }
    if showing.search.holding_back() {
        parts.push(format!("matching {:?}", showing.search.query()));
    }
    if arrangement == Arrangement::ByProgram {
        parts.push("grouped by program".to_string());
    }
    if !showing.sorting.as_read() {
        parts.push(format!(
            "sorted by {}",
            showing.sorting.describe(super::SORTED_BY)
        ));
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
