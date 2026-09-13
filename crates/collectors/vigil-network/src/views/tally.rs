use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

pub(super) fn tally(
    reading: &Snapshot,
    showing: &Showing<'_>,
    shown: usize,
    grouped: bool,
) -> String {
    let unresolved = reading
        .items
        .values()
        .filter(|item| item.get("owner_resolved").and_then(Value::as_bool) == Some(false))
        .count();

    let mut parts = vec![match showing.narrowed() {
        false => format!(
            "{} listening socket(s), read at {}",
            reading.items.len(),
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{} of {} listening socket(s), read at {}",
            shown,
            reading.items.len(),
            time_of_day(&reading.taken_at)
        ),
    }];
    if unresolved > 0 {
        parts.push(format!("{unresolved} whose process is not visible"));
    }
    if !showing.hidden.is_empty() {
        parts.push(format!("hiding {}", showing.hidden.join(", ")));
    }
    if showing.holding_back() {
        parts.push(format!("matching {:?}", showing.search));
    }
    if grouped {
        parts.push("grouped by program".to_string());
    }

    parts.join(" · ")
}
