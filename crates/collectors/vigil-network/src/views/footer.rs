use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Counts, Rows, Showing, time_of_day};

const NOT_VISIBLE: &str = "sockets whose process is not visible";

pub(super) fn counts(reading: &Snapshot) -> Counts {
    let unresolved = reading
        .items
        .values()
        .filter(|item| item.get("owner_resolved").and_then(Value::as_bool) == Some(false))
        .count();
    Counts::default().counted(NOT_VISIBLE, unresolved)
}

pub(super) fn tallied(
    reading: &Snapshot,
    showing: &Showing<'_>,
    rows: &Rows<'_>,
    counts: &Counts,
    grouped: bool,
) -> String {
    let read_at = time_of_day(&reading.taken_at);
    let whole = reading.items.len();

    let mut parts = vec![match showing.narrowed() {
        false => format!("{whole} listening socket(s), read at {read_at}"),
        true => format!(
            "{} of {whole} listening socket(s), read at {read_at}",
            rows.len()
        ),
    }];
    let unresolved = counts.number(NOT_VISIBLE);
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
