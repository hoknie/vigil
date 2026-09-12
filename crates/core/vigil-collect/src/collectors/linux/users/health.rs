use std::fs;
use std::io::ErrorKind;

use super::sessions::read_sessions;
use super::{PASSWD, SHADOW, SUDOERS};
use crate::Health;
use crate::parsers::SessionSource;

pub fn health() -> Health {
    if fs::read_to_string(PASSWD).is_err() {
        return Health::Unavailable(format!("{PASSWD} cannot be read"));
    }

    let mut missing: Vec<String> = Vec::new();

    if let Err(error) = fs::read_to_string(SHADOW) {
        missing.push(match error.kind() {
            ErrorKind::PermissionDenied => format!(
                "{SHADOW} is not readable: lock state and password dates will be missing; run as root"
            ),
            _ => format!("{SHADOW} is not present: lock state and password dates will be missing"),
        });
    }
    match fs::read_to_string(SUDOERS) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) if error.kind() == ErrorKind::PermissionDenied => missing.push(format!(
            "{SUDOERS} is not readable: grants made there will not be seen; run as root"
        )),
        Err(error) => missing.push(format!(
            "{SUDOERS} could not be read ({error}): grants made there will not be seen"
        )),
    }
    missing.extend(session_notes(&read_sessions(&[]).sources));

    match missing.is_empty() {
        true => Health::Ok,
        false => Health::Degraded(missing.join("; ")),
    }
}

pub(super) fn session_notes(sources: &[SessionSource]) -> Vec<String> {
    if sources.iter().any(SessionSource::answers) {
        return sources
            .iter()
            .filter(|source| source.present && !source.read)
            .filter_map(|source| source.reason.clone())
            .map(|reason| format!("{reason}: sessions it knows about will be missing"))
            .collect();
    }

    vec![format!(
        "no record of logins on this host ({}): who is logged in will be missing",
        sources.iter().map(shortly).collect::<Vec<_>>().join(", ")
    )]
}

fn shortly(source: &SessionSource) -> String {
    match (source.present, source.read) {
        (false, _) => format!("no {}", source.path),
        (true, false) => format!("{} could not be read", source.path),
        (true, true) => format!("{} records none", source.path),
    }
}
