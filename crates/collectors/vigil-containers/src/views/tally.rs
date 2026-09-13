use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

use super::fields::{SYS_ADMIN, capabilities, mounts_readable, text, truncated};
use crate::types::Kind;

pub(super) fn tally(reading: &Snapshot, showing: &Showing<'_>, shown: usize) -> String {
    let whole = reading
        .items
        .keys()
        .filter(|key| Kind::of(key) == Some(Kind::Container))
        .count();

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{shown} container(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {whole} container(s), read at {}",
            time_of_day(&reading.taken_at)
        ),
    }];

    parts.push(match sockets(reading).as_slice() {
        [] => "no runtime socket on this host".to_string(),
        held => held
            .iter()
            .map(|(path, mode)| format!("socket {path} {mode}"))
            .collect::<Vec<String>>()
            .join(", "),
    });

    let containers: Vec<&Value> = reading
        .items
        .iter()
        .filter(|(key, _)| Kind::of(key) == Some(Kind::Container))
        .map(|(_, item)| item)
        .collect();

    let privileged = containers
        .iter()
        .filter(|item| capabilities(item).is_some_and(|held| held & SYS_ADMIN != 0))
        .count();
    if privileged > 0 {
        parts.push(format!("{privileged} holding SYS_ADMIN"));
    }

    let unreadable = containers
        .iter()
        .filter(|item| capabilities(item).is_none() || !mounts_readable(item))
        .count();
    if unreadable > 0 {
        parts.push(format!("{unreadable} could not be read whole"));
    }
    if containers.iter().any(|item| truncated(item)) {
        parts.push("one holds more host paths than this reading carries".to_string());
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" · ")
}

fn sockets(reading: &Snapshot) -> Vec<(String, String)> {
    reading
        .items
        .iter()
        .filter(|(key, _)| Kind::of(key) == Some(Kind::Socket))
        .map(|(_, item)| {
            (
                text(item, "path").to_string(),
                text(item, "mode").to_string(),
            )
        })
        .collect()
}
