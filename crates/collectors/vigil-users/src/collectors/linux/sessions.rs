use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use vigil_collect::PasswdEntry;

use crate::parsers::{
    LOGIND, Session, SessionSource, UTMP, is_session_file, merge_sessions, parse_logind_session,
    parse_utmp,
};

const UTMP_PATHS: &[&str] = &["/run/utmp", "/var/run/utmp"];

const LOGIND_SESSIONS: &str = "/run/systemd/sessions";

const SESSION_FILE_LIMIT: usize = 4096;

pub struct SessionReading {
    pub sessions: Vec<Session>,
    pub sources: Vec<SessionSource>,
}

pub fn read_sessions(passwd: &[PasswdEntry]) -> SessionReading {
    let mut records = Vec::new();
    let mut sources = Vec::new();

    for (found, source) in [read_utmp(), read_logind()] {
        records.extend(found);
        sources.push(source);
    }

    for record in &mut records {
        if !record.user.is_empty() {
            continue;
        }
        if let Some(uid) = record.uid
            && let Some(entry) = passwd.iter().find(|entry| entry.uid == uid)
        {
            record.user = entry.name.clone();
        }
    }

    SessionReading {
        sessions: merge_sessions(records),
        sources,
    }
}

fn read_utmp() -> (Vec<Session>, SessionSource) {
    let Some(path) = UTMP_PATHS
        .iter()
        .copied()
        .find(|path| fs::metadata(path).is_ok())
    else {
        return (
            Vec::new(),
            SessionSource::absent(
                UTMP,
                UTMP_PATHS[0],
                format!(
                    "neither {} nor {} is on this host: nothing writes a utmp login record here",
                    UTMP_PATHS[0], UTMP_PATHS[1]
                ),
            ),
        );
    };

    match fs::read(path) {
        Err(error) => (
            Vec::new(),
            SessionSource::refused(UTMP, path, format!("{path} could not be read: {error}")),
        ),
        Ok(bytes) if bytes.is_empty() => (
            Vec::new(),
            SessionSource::read(UTMP, path, 0).saying(format!(
                "{path} is there and empty: on this host nothing writes logins to it"
            )),
        ),
        Ok(bytes) => match parse_utmp(&bytes) {
            Some(sessions) => {
                let held = sessions.len();
                (sessions, SessionSource::read(UTMP, path, held))
            }
            None => (
                Vec::new(),
                SessionSource::refused(
                    UTMP,
                    path,
                    format!("{path} is not in the shape this build reads"),
                ),
            ),
        },
    }
}

fn read_logind() -> (Vec<Session>, SessionSource) {
    let directory = Path::new(LOGIND_SESSIONS);
    let listing = match fs::read_dir(directory) {
        Ok(listing) => listing,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return (
                Vec::new(),
                SessionSource::absent(
                    LOGIND,
                    LOGIND_SESSIONS,
                    format!(
                        "{LOGIND_SESSIONS} is not there: no systemd-logind is keeping sessions on this host"
                    ),
                ),
            );
        }
        Err(error) => {
            return (
                Vec::new(),
                SessionSource::refused(
                    LOGIND,
                    LOGIND_SESSIONS,
                    format!("{LOGIND_SESSIONS} could not be listed: {error}"),
                ),
            );
        }
    };

    let mut names: Vec<String> = listing
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| is_session_file(name))
        .take(SESSION_FILE_LIMIT)
        .collect();
    names.sort();

    let mut sessions = Vec::new();
    let mut refused = 0usize;
    for name in &names {
        match fs::read_to_string(directory.join(name)) {
            Ok(text) => sessions.extend(parse_logind_session(name, &text)),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(_) => refused += 1,
        }
    }

    let held = sessions.len();
    let source = SessionSource::read(LOGIND, LOGIND_SESSIONS, held);
    match refused {
        0 => (sessions, source),
        _ => (
            sessions,
            source.saying(format!(
                "{refused} of the {} session files under {LOGIND_SESSIONS} could not be read; run as root",
                names.len()
            )),
        ),
    }
}
