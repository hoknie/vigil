use serde_json::Value;
use vigil_model::Snapshot;

use super::facts::{attended, could_log_in, privileged, readable};
use super::fields::objects;
use super::fields::text;
use super::kind::Kind;
use super::rows::rows;
use super::showing::Showing;
use crate::ui::helpers::words::moment;
use crate::ui::{Search, Subject, View};

pub(super) fn tally(
    view: &View,
    showing: &Showing<'_>,
    snapshot: &Snapshot,
    shown: usize,
    width: u16,
) -> String {
    let subject = showing.subject;
    let counted = |search: &Search| {
        rows(view, subject, search)
            .iter()
            .filter(|row| row.kind != Kind::SessionSource)
            .count()
    };
    let held = counted(&Search::default());
    let shown = match showing.search.holding_back() {
        true => counted(showing.search),
        false => shown,
    };

    let mut parts = vec![match showing.search.holding_back() {
        false => format!(
            "{held} {}, read at {}",
            subject.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
        true => format!(
            "{shown} of {held} {}, read at {}",
            subject.things(held),
            moment::time_of_day(&snapshot.taken_at)
        ),
    }];

    match subject {
        Subject::Users => {
            let usable = objects(view, Kind::Account)
                .filter(|(_, account)| could_log_in(account))
                .count();
            parts.push(format!("{usable} could log in"));
            let opaque = objects(view, Kind::Account)
                .filter(|(_, account)| {
                    account.get("shadow_readable").and_then(Value::as_bool) != Some(true)
                })
                .count();
            if opaque > 0 {
                parts.push(format!("{opaque} whose password state could not be read"));
            }
        }
        Subject::Groups => {
            let privileged = objects(view, Kind::Group)
                .filter(|(_, group)| privileged(group))
                .count();
            parts.push(format!("{privileged} that grant power"));
        }
        Subject::Keys | Subject::SshUsers => {
            let refused = objects(view, Kind::Key)
                .filter(|(_, key)| !readable(key))
                .count();
            if refused > 0 {
                parts.push(format!("{refused} key file(s) the agent was refused"));
            }
        }
        Subject::LoggedIn => {
            let watched = objects(view, Kind::Session)
                .filter(|(_, session)| attended(session))
                .count();
            parts.push(format!("{watched} with somebody at a terminal"));
            for (_, source) in objects(view, Kind::SessionSource) {
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
    if let Some(reason) = view.collector_note("users") {
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
