#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    pub user: String,
    pub line: String,
    pub from: String,
    pub pid: u32,
}

impl Session {
    pub fn is_remote(&self) -> bool {
        !self.from.is_empty()
    }
}

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

        sessions.push(Session {
            user,
            line: text(&record[OFFSET_LINE..OFFSET_LINE + 32]),
            from: text(&record[OFFSET_HOST..OFFSET_END_OF_HOST]),
            pid: u32::from_ne_bytes([
                record[OFFSET_PID],
                record[OFFSET_PID + 1],
                record[OFFSET_PID + 2],
                record[OFFSET_PID + 3],
            ]),
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
mod tests {
    use super::*;

    fn record(kind: i16, pid: u32, line: &str, user: &str, host: &str) -> Vec<u8> {
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

    #[test]
    fn reads_the_live_logins_and_leaves_the_tombstones_alone() {
        let mut file = record(USER_PROCESS, 4021, "pts/0", "deploy", "10.0.0.7");
        file.extend(record(USER_PROCESS, 900, "tty1", "root", ""));
        file.extend(record(8, 3311, "pts/1", "alice", "10.0.0.9"));
        file.extend(record(2, 0, "~", "reboot", "6.6.0"));

        let sessions = parse_utmp(&file).expect("a utmp file");

        assert_eq!(sessions.len(), 2, "{sessions:?}");
        assert_eq!(sessions[0].user, "deploy");
        assert_eq!(sessions[0].line, "pts/0");
        assert_eq!(sessions[0].from, "10.0.0.7");
        assert_eq!(sessions[0].pid, 4021);
        assert!(sessions[0].is_remote());
        assert!(
            !sessions[1].is_remote(),
            "a console login has no source address"
        );
    }

    #[test]
    fn a_file_in_a_shape_we_do_not_know_is_not_an_empty_host() {
        assert!(parse_utmp(&[0u8; 100]).is_none());
        assert!(parse_utmp(b"not a utmp file").is_none());
        assert_eq!(parse_utmp(&[]), Some(Vec::new()));
    }
}
