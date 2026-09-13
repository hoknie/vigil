use std::collections::BTreeSet;

use super::session::{Session, UTMP};

const RECORD: usize = 384;

const USER_PROCESS: i16 = 7;

const OFFSET_PID: usize = 4;
const OFFSET_LINE: usize = 8;
const OFFSET_USER: usize = 44;
const OFFSET_HOST: usize = 76;
const OFFSET_END_OF_HOST: usize = 332;

pub fn parse_utmp(bytes: &[u8]) -> Option<Vec<Session>> {
    if !bytes.len().is_multiple_of(RECORD) {
        return None;
    }

    let mut sessions = Vec::new();
    for record in bytes.as_chunks::<RECORD>().0 {
        let kind = i16::from_ne_bytes([record[0], record[1]]);
        if kind != USER_PROCESS {
            continue;
        }

        let user = text(&record[OFFSET_USER..OFFSET_HOST]);
        if user.is_empty() {
            continue;
        }

        let from = text(&record[OFFSET_HOST..OFFSET_END_OF_HOST]);
        sessions.push(Session {
            user,
            line: text(&record[OFFSET_LINE..OFFSET_LINE + 32]),
            remote: !from.is_empty(),
            from,
            pid: u32::from_ne_bytes([
                record[OFFSET_PID],
                record[OFFSET_PID + 1],
                record[OFFSET_PID + 2],
                record[OFFSET_PID + 3],
            ]),
            sources: BTreeSet::from([UTMP]),
            ..Session::default()
        });
    }

    Some(sessions)
}

fn text(field: &[u8]) -> String {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end])
        .chars()
        .filter(|c| !c.is_control())
        .collect()
}

#[cfg(test)]
pub(super) fn record(kind: i16, pid: u32, line: &str, user: &str, host: &str) -> Vec<u8> {
    let mut buffer = vec![0u8; RECORD];
    buffer[0..2].copy_from_slice(&kind.to_ne_bytes());
    buffer[OFFSET_PID..OFFSET_PID + 4].copy_from_slice(&pid.to_ne_bytes());
    for (offset, value) in [
        (OFFSET_LINE, line),
        (OFFSET_USER, user),
        (OFFSET_HOST, host),
    ] {
        buffer[offset..offset + value.len()].copy_from_slice(value.as_bytes());
    }
    buffer
}

#[cfg(test)]
pub(super) const LIVE: i16 = USER_PROCESS;
