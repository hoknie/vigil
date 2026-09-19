use std::collections::BTreeSet;
use std::fs;
use std::io::ErrorKind;
use std::sync::Mutex;

use crate::parsers::{Session, SessionSource, UTMP};

pub(super) const LOGINS: &str = "/var/run/utmpx";

const MOST_LOGINS: usize = 4096;

static ENUMERATING: Mutex<()> = Mutex::new(());

pub(super) fn read_logins() -> (Vec<Session>, SessionSource) {
    match fs::metadata(LOGINS) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return (
                Vec::new(),
                SessionSource::absent(
                    UTMP,
                    LOGINS,
                    format!("{LOGINS} is not on this host: nothing writes a login record here"),
                ),
            );
        }
        Err(error) => {
            return (
                Vec::new(),
                SessionSource::refused(
                    UTMP,
                    LOGINS,
                    format!("{LOGINS} could not be read: {error}"),
                ),
            );
        }
    }

    let sessions = logins();
    let held = sessions.len();
    (sessions, SessionSource::read(UTMP, LOGINS, held))
}

fn logins() -> Vec<Session> {
    let _alone = ENUMERATING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut sessions = Vec::new();

    unsafe { libc::setutxent() };
    while sessions.len() < MOST_LOGINS {
        let entry = unsafe { libc::getutxent() };
        if entry.is_null() {
            break;
        }
        if let Some(session) = session_of(unsafe { &*entry }) {
            sessions.push(session);
        }
    }
    unsafe { libc::endutxent() };

    sessions
}

fn session_of(entry: &libc::utmpx) -> Option<Session> {
    if entry.ut_type != libc::USER_PROCESS {
        return None;
    }
    let user = text(&entry.ut_user);
    if user.is_empty() {
        return None;
    }
    let from = text(&entry.ut_host);

    Some(Session {
        user,
        line: text(&entry.ut_line),
        remote: !from.is_empty(),
        from,
        pid: u32::try_from(entry.ut_pid).unwrap_or(0),
        sources: BTreeSet::from([UTMP]),
        ..Session::default()
    })
}

fn text(field: &[libc::c_char]) -> String {
    let bytes: Vec<u8> = field
        .iter()
        .take_while(|character| **character != 0)
        .map(|character| character.cast_unsigned())
        .collect();
    String::from_utf8_lossy(&bytes)
        .chars()
        .filter(|character| !character.is_control())
        .collect()
}
