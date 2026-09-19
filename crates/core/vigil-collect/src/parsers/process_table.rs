use crate::types::ProcessEntry;

pub const PROCESS_ENTRY_BYTES: usize = 648;

const STATE: usize = 36;

const PID: usize = 40;

const NAME: usize = 243;

const NAME_BYTES: usize = 17;

const REAL_UID: usize = 392;

const EFFECTIVE_UID: usize = 420;

const PARENT: usize = 560;

const ZOMBIE: u8 = 5;

pub fn parse_process_table(bytes: &[u8]) -> Option<Vec<ProcessEntry>> {
    if !bytes.len().is_multiple_of(PROCESS_ENTRY_BYTES) {
        return None;
    }

    bytes
        .as_chunks::<PROCESS_ENTRY_BYTES>()
        .0
        .iter()
        .map(|entry| entry_of(entry))
        .collect()
}

fn entry_of(entry: &[u8]) -> Option<ProcessEntry> {
    Some(ProcessEntry {
        pid: u32::try_from(signed(entry, PID)?).ok()?,
        parent: u32::try_from(signed(entry, PARENT)?).unwrap_or(0),
        real_uid: unsigned(entry, REAL_UID)?,
        effective_uid: unsigned(entry, EFFECTIVE_UID)?,
        name: text(entry.get(NAME..NAME + NAME_BYTES)?),
        zombie: *entry.get(STATE)? == ZOMBIE,
    })
}

fn signed(entry: &[u8], at: usize) -> Option<i32> {
    Some(i32::from_ne_bytes(entry.get(at..at + 4)?.try_into().ok()?))
}

fn unsigned(entry: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_ne_bytes(entry.get(at..at + 4)?.try_into().ok()?))
}

fn text(field: &[u8]) -> String {
    let end = field
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(field.len());
    String::from_utf8_lossy(&field[..end]).into_owned()
}

#[cfg(test)]
pub(crate) fn sample_entry(
    pid: i32,
    parent: i32,
    real_uid: u32,
    effective_uid: u32,
    name: &str,
    state: u8,
) -> Vec<u8> {
    let mut entry = vec![0u8; PROCESS_ENTRY_BYTES];
    entry[STATE] = state;
    entry[PID..PID + 4].copy_from_slice(&pid.to_ne_bytes());
    entry[PARENT..PARENT + 4].copy_from_slice(&parent.to_ne_bytes());
    entry[REAL_UID..REAL_UID + 4].copy_from_slice(&real_uid.to_ne_bytes());
    entry[EFFECTIVE_UID..EFFECTIVE_UID + 4].copy_from_slice(&effective_uid.to_ne_bytes());
    entry[NAME..NAME + name.len()].copy_from_slice(name.as_bytes());
    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_entry_of_the_table_is_read_with_its_parent_and_both_of_its_accounts() {
        let mut table = sample_entry(812, 1, 0, 0, "nginx", 2);
        table.extend(sample_entry(900, 812, 501, 0, "sudo", 2));

        let entries = parse_process_table(&table).expect("two whole entries");

        assert_eq!(
            entries,
            vec![
                ProcessEntry {
                    pid: 812,
                    parent: 1,
                    real_uid: 0,
                    effective_uid: 0,
                    name: "nginx".into(),
                    zombie: false,
                },
                ProcessEntry {
                    pid: 900,
                    parent: 812,
                    real_uid: 501,
                    effective_uid: 0,
                    name: "sudo".into(),
                    zombie: false,
                },
            ]
        );
    }

    #[test]
    fn a_process_that_has_exited_and_waits_for_its_parent_is_marked_as_such() {
        let entries =
            parse_process_table(&sample_entry(4242, 812, 33, 33, "worker", 5)).expect("whole");

        assert!(entries[0].zombie);
    }

    #[test]
    fn a_table_cut_mid_entry_is_refused_rather_than_read_as_fewer_processes() {
        let mut table = sample_entry(812, 1, 0, 0, "nginx", 2);
        table.extend(&sample_entry(900, 812, 501, 501, "zsh", 2)[..100]);

        assert_eq!(
            parse_process_table(&table),
            None,
            "a reading that silently drops the last process is a host with one program fewer"
        );
        assert_eq!(parse_process_table(&[]), Some(Vec::new()));
    }

    #[test]
    fn a_name_that_fills_its_whole_field_is_read_to_the_end_of_the_field() {
        let entries =
            parse_process_table(&sample_entry(1, 0, 0, 0, "abcdefghijklmnopq", 2)).expect("whole");

        assert_eq!(entries[0].name, "abcdefghijklmnopq");
    }

    #[test]
    fn the_kernel_whose_parent_is_nobody_is_read_with_no_parent() {
        let entries =
            parse_process_table(&sample_entry(0, -1, 0, 0, "kernel_task", 2)).expect("whole");

        assert_eq!(entries[0].pid, 0);
        assert_eq!(entries[0].parent, 0);
    }
}
