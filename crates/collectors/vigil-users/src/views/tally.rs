use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Showing, time_of_day};

use super::facts::{attended, could_log_in, privileged, readable};
use super::fields::{objects, text};
use super::rows::rows;
use crate::types::{Kind, Subject};

pub(super) fn tally(
    reading: &Snapshot,
    subject: Subject,
    showing: &Showing<'_>,
    shown: usize,
) -> String {
    let counted = |showing: &Showing<'_>| {
        rows(reading, subject, showing)
            .iter()
            .filter(|row| Kind::of(&row.key) != Kind::SessionSource)
            .count()
    };
    let held = counted(&Showing::default());
    let shown = match showing.holding_back() {
        true => counted(showing),
        false => shown,
    };

    let mut parts = vec![match showing.holding_back() {
        false => format!(
            "{held} {}, read at {}",
            subject.things(held),
            time_of_day(&reading.taken_at)
        ),
        true => format!(
            "{shown} of {held} {}, read at {}",
            subject.things(held),
            time_of_day(&reading.taken_at)
        ),
    }];

    match subject {
        Subject::Users => {
            let usable = objects(reading, Kind::Account)
                .filter(|(_, account)| could_log_in(account))
                .count();
            parts.push(format!("{usable} could log in"));
            let opaque = objects(reading, Kind::Account)
                .filter(|(_, account)| {
                    account.get("shadow_readable").and_then(Value::as_bool) != Some(true)
                })
                .count();
            if opaque > 0 {
                parts.push(format!("{opaque} whose password state could not be read"));
            }
        }
        Subject::Groups => {
            let privileged = objects(reading, Kind::Group)
                .filter(|(_, group)| privileged(group))
                .count();
            parts.push(format!("{privileged} that grant power"));
        }
        Subject::Keys | Subject::SshUsers => {
            let refused = objects(reading, Kind::Key)
                .filter(|(_, key)| !readable(key))
                .count();
            if refused > 0 {
                parts.push(format!("{refused} key file(s) the agent was refused"));
            }
        }
        Subject::LoggedIn => {
            let watched = objects(reading, Kind::Session)
                .filter(|(_, session)| attended(session))
                .count();
            parts.push(format!("{watched} with somebody at a terminal"));
            for (_, source) in objects(reading, Kind::SessionSource) {
                parts.push(standing(source));
            }
        }
        Subject::Sudo | Subject::Other => {}
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

fn standing(source: &Value) -> String {
    let name = text(source, "source").unwrap_or("?");
    match (
        source.get("present").and_then(Value::as_bool),
        source.get("read").and_then(Value::as_bool),
    ) {
        (Some(false), _) => format!("{name}: not on this host"),
        (_, Some(false)) => format!("{name}: on this host and not read"),
        _ => format!(
            "{name}: read, {}",
            source
                .get("sessions")
                .and_then(Value::as_u64)
                .unwrap_or_default()
        ),
    }
}
