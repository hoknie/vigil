use vigil_model::Snapshot;
use vigil_view::{Counts, Rows, Showing, time_of_day};

use super::fields::{SYS_ADMIN, capabilities, mounts_readable, text, truncated};
use crate::types::Kind;

const CONTAINERS: &str = "containers";
const PRIVILEGED: &str = "holding SYS_ADMIN";
const UNREADABLE: &str = "could not be read whole";
const TRUNCATED: &str = "holding more host paths than the reading carries";
const SOCKETS: &str = "runtime sockets";

pub(super) fn counts(reading: &Snapshot) -> Counts {
    let mut counts = [0; 4];
    let mut sockets = Vec::new();
    for (key, item) in &reading.items {
        match Kind::of(key) {
            Some(Kind::Container) => {
                let held = capabilities(item);
                counts[0] += 1;
                counts[1] += usize::from(held.is_some_and(|held| held & SYS_ADMIN != 0));
                counts[2] += usize::from(held.is_none() || !mounts_readable(item));
                counts[3] += usize::from(truncated(item));
            }
            Some(Kind::Socket) => sockets.push(format!(
                "socket {} {}",
                text(item, "path"),
                text(item, "mode")
            )),
            None => {}
        }
    }

    Counts::default()
        .counted(CONTAINERS, counts[0])
        .counted(PRIVILEGED, counts[1])
        .counted(UNREADABLE, counts[2])
        .counted(TRUNCATED, counts[3])
        .saying(SOCKETS, sockets)
}

pub(super) fn tallied(
    reading: &Snapshot,
    showing: &Showing<'_>,
    rows: &Rows<'_>,
    counts: &Counts,
) -> String {
    let shown = rows.len();
    let read_at = time_of_day(&reading.taken_at);

    let mut parts = vec![match showing.holding_back() {
        false => format!("{shown} container(s), read at {read_at}"),
        true => format!(
            "{shown} of {} container(s), read at {read_at}",
            counts.number(CONTAINERS)
        ),
    }];
    parts.push(match counts.words(SOCKETS) {
        [] => "no runtime socket on this host".to_string(),
        held => held.join(", "),
    });
    let privileged = counts.number(PRIVILEGED);
    if privileged > 0 {
        parts.push(format!("{privileged} holding SYS_ADMIN"));
    }
    let unreadable = counts.number(UNREADABLE);
    if unreadable > 0 {
        parts.push(format!("{unreadable} could not be read whole"));
    }
    if counts.number(TRUNCATED) > 0 {
        parts.push("one holds more host paths than this reading carries".to_string());
    }
    if let Some(reason) = showing.note {
        parts.push(format!("incomplete: {reason}"));
    }

    parts.join(" · ")
}
