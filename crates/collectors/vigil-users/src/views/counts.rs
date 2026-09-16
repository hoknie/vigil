use serde_json::Value;
use vigil_model::Snapshot;
use vigil_view::{Counts, RowKey, Rows, Showing, time_of_day};

use super::facts::{attended, could_log_in, privileged, readable};
use super::fields::objects;
use super::rows::rows;
use super::tally::standing;
use crate::types::{Kind, Subject};

const HELD: &str = "held";
const COULD_LOG_IN: &str = "could log in";
const OPAQUE: &str = "opaque passwords";
const PRIVILEGED: &str = "privileged groups";
const REFUSED: &str = "refused key files";
const ATTENDED: &str = "attended sessions";
const SOURCES: &str = "session sources";

fn not_a_source(row: &RowKey) -> bool {
    Kind::of(&row.key) != Kind::SessionSource
}

pub(super) fn counts(reading: &Snapshot, subject: Subject) -> Counts {
    let held = rows(reading, subject, &Showing::default())
        .iter()
        .filter(|row| not_a_source(row))
        .count();
    let counts = Counts::default().counted(HELD, held);

    match subject {
        Subject::Users => counts
            .counted(
                COULD_LOG_IN,
                objects(reading, Kind::Account)
                    .filter(|(_, account)| could_log_in(account))
                    .count(),
            )
            .counted(
                OPAQUE,
                objects(reading, Kind::Account)
                    .filter(|(_, account)| {
                        account.get("shadow_readable").and_then(Value::as_bool) != Some(true)
                    })
                    .count(),
            ),
        Subject::Groups => counts.counted(
            PRIVILEGED,
            objects(reading, Kind::Group)
                .filter(|(_, group)| privileged(group))
                .count(),
        ),
        Subject::Keys | Subject::SshUsers => counts.counted(
            REFUSED,
            objects(reading, Kind::Key)
                .filter(|(_, key)| !readable(key))
                .count(),
        ),
        Subject::LoggedIn => counts
            .counted(
                ATTENDED,
                objects(reading, Kind::Session)
                    .filter(|(_, session)| attended(session))
                    .count(),
            )
            .saying(
                SOURCES,
                objects(reading, Kind::SessionSource)
                    .map(|(_, source)| standing(source))
                    .collect(),
            ),
        Subject::Sudo | Subject::Other => counts,
    }
}

pub(super) fn tally_listed(
    reading: &Snapshot,
    subject: Subject,
    showing: &Showing<'_>,
    listed: &Rows<'_>,
    counts: &Counts,
) -> String {
    let held = counts.number(HELD);
    let read_at = time_of_day(&reading.taken_at);

    let mut parts = vec![match showing.holding_back() {
        false => format!("{held} {}, read at {read_at}", subject.things(held)),
        true => format!(
            "{} of {held} {}, read at {read_at}",
            listed.iter().filter(|row| not_a_source(row)).count(),
            subject.things(held)
        ),
    }];

    match subject {
        Subject::Users => {
            parts.push(format!("{} could log in", counts.number(COULD_LOG_IN)));
            let opaque = counts.number(OPAQUE);
            if opaque > 0 {
                parts.push(format!("{opaque} whose password state could not be read"));
            }
        }
        Subject::Groups => {
            parts.push(format!("{} that grant power", counts.number(PRIVILEGED)));
        }
        Subject::Keys | Subject::SshUsers => {
            let refused = counts.number(REFUSED);
            if refused > 0 {
                parts.push(format!("{refused} key file(s) the agent was refused"));
            }
        }
        Subject::LoggedIn => {
            parts.push(format!(
                "{} with somebody at a terminal",
                counts.number(ATTENDED)
            ));
            parts.extend(counts.words(SOURCES).iter().cloned());
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
