use std::io;
use std::sync::OnceLock;

use super::sysctl::{sysctl_into, sysctl_named, sysctl_numbered};
use crate::parsers::parse_process_table;
use crate::types::{CollectError, ProcessEntry};

const PATH_BYTES: usize = 4 * libc::MAXPATHLEN as usize;

const ARGUMENTS_CEILING: usize = 1024 * 1024;

pub fn process_table() -> io::Result<Vec<u8>> {
    sysctl_numbered(&[libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_ALL])
}

pub fn processes_running() -> Result<Vec<ProcessEntry>, CollectError> {
    let bytes = process_table()
        .map_err(|error| CollectError::Unreadable(format!("the table of processes: {error}")))?;
    let entries = parse_process_table(&bytes).ok_or_else(|| {
        CollectError::Unreadable(
            "the table of processes, which is in a shape this build does not know".into(),
        )
    })?;

    match entries.is_empty() {
        true => Err(CollectError::Unreadable(
            "the table of processes, which holds no process".into(),
        )),
        false => Ok(entries),
    }
}

pub fn process_entry(pid: u32) -> io::Result<Vec<u8>> {
    let pid = libc::c_int::try_from(pid).map_err(|_| io::Error::from_raw_os_error(libc::ESRCH))?;
    sysctl_numbered(&[libc::CTL_KERN, libc::KERN_PROC, libc::KERN_PROC_PID, pid])
}

pub fn executable_of(pid: u32) -> io::Result<String> {
    let pid = libc::c_int::try_from(pid).map_err(|_| io::Error::from_raw_os_error(libc::ESRCH))?;
    let mut buffer = vec![0u8; PATH_BYTES];

    let written = unsafe {
        libc::proc_pidpath(
            pid,
            buffer.as_mut_ptr().cast(),
            u32::try_from(buffer.len()).unwrap_or(u32::MAX),
        )
    };
    if written <= 0 {
        return Err(io::Error::last_os_error());
    }

    buffer.truncate(usize::try_from(written).unwrap_or(0));
    Ok(String::from_utf8_lossy(&buffer).into_owned())
}

pub fn arguments_buffer() -> Vec<u8> {
    vec![0u8; largest_arguments()]
}

pub fn arguments_of(pid: u32, buffer: &mut [u8]) -> io::Result<usize> {
    let pid = libc::c_int::try_from(pid).map_err(|_| io::Error::from_raw_os_error(libc::ESRCH))?;
    sysctl_into(&[libc::CTL_KERN, libc::KERN_PROCARGS2, pid], buffer)
}

fn largest_arguments() -> usize {
    static LARGEST: OnceLock<usize> = OnceLock::new();

    *LARGEST.get_or_init(|| {
        sysctl_named("kern.argmax")
            .ok()
            .and_then(|bytes| Some(i32::from_ne_bytes(bytes.get(..4)?.try_into().ok()?)))
            .and_then(|largest| usize::try_from(largest).ok())
            .unwrap_or(ARGUMENTS_CEILING)
            .clamp(4096, ARGUMENTS_CEILING)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::parse_process_arguments;

    #[test]
    fn the_table_of_every_process_holds_this_one_with_its_parent_and_its_account() {
        let table = parse_process_table(&process_table().expect("read")).expect("whole entries");
        let me = std::process::id();

        let entry = table
            .iter()
            .find(|entry| entry.pid == me)
            .expect("this test is running");

        assert_eq!(entry.real_uid, unsafe { libc::getuid() });
        assert_eq!(entry.effective_uid, unsafe { libc::geteuid() });
        assert_eq!(entry.parent, std::os::unix::process::parent_id());
        assert!(!entry.zombie);
        assert!(
            table.iter().any(|entry| entry.pid == 1),
            "launchd is in the table of every account, which is what makes the table the \
             list of processes rather than the list of this account's own"
        );
    }

    #[test]
    fn the_processes_running_are_read_as_entries_and_this_one_is_among_them() {
        let entries = processes_running().expect("read");

        assert!(entries.iter().any(|entry| entry.pid == std::process::id()));
    }

    #[test]
    fn one_process_is_read_as_the_one_entry_of_the_same_table() {
        let me = std::process::id();

        let table = parse_process_table(&process_entry(me).expect("read")).expect("whole");

        assert_eq!(table.len(), 1);
        assert_eq!(table[0].pid, me);
        assert!(
            parse_process_table(&process_entry(99_999_999).unwrap_or_default())
                .unwrap_or_default()
                .is_empty()
        );
    }

    #[test]
    fn the_program_of_this_process_is_the_file_it_was_started_from() {
        let expected = std::env::current_exe().expect("the test binary");
        let expected = std::fs::canonicalize(expected).expect("canonical");

        let path = executable_of(std::process::id()).expect("its own program");

        assert_eq!(path, expected.to_string_lossy());
    }

    #[test]
    fn a_process_that_does_not_exist_has_no_program_and_says_why() {
        let error = executable_of(99_999_999).expect_err("no such process");

        assert_eq!(error.raw_os_error(), Some(libc::ESRCH));
    }

    #[test]
    fn the_arguments_of_this_process_are_read_from_the_block_the_kernel_keeps() {
        let mut buffer = arguments_buffer();

        let written = arguments_of(std::process::id(), &mut buffer).expect("its own");
        let read = parse_process_arguments(&buffer[..written]).expect("parses");

        assert_eq!(read.arguments, std::env::args().collect::<Vec<_>>());
    }

    #[test]
    fn the_arguments_of_a_process_of_the_superuser_are_not_shown_to_another_account() {
        if unsafe { libc::geteuid() } == 0 {
            return;
        }
        let mut buffer = arguments_buffer();

        assert!(
            arguments_of(1, &mut buffer).is_err(),
            "macOS shows the arguments of a process to its own account and to root, and \
             every reading has to count what it was not shown"
        );
    }
}
