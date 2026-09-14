use std::fs;

use crate::parsers::parse_status;

pub fn running(executable: &str, uid: u32) -> Result<Vec<u32>, String> {
    let entries =
        fs::read_dir("/proc").map_err(|error| format!("/proc cannot be listed: {error}"))?;

    let mut pids: Vec<u32> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_str()?.parse::<u32>().ok())
        .filter(|pid| runs(*pid, executable, uid))
        .collect();
    pids.sort_unstable();
    Ok(pids)
}

fn runs(pid: u32, executable: &str, uid: u32) -> bool {
    still_running(pid, executable, Some(uid))
}

pub fn still_running(pid: u32, executable: &str, uid: Option<u32>) -> bool {
    if let Some(uid) = uid {
        let Ok(text) = fs::read_to_string(format!("/proc/{pid}/status")) else {
            return false;
        };
        if parse_status(&text).is_none_or(|status| status.uid != uid) {
            return false;
        }
    }
    let Ok(link) = fs::read_link(format!("/proc/{pid}/exe")) else {
        return false;
    };

    same_program(&link.to_string_lossy(), executable)
}

fn same_program(link: &str, executable: &str) -> bool {
    link.strip_suffix(" (deleted)").unwrap_or(link) == executable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_test_is_found_among_the_processes_running_its_own_program_as_its_own_account() {
        let executable = fs::read_link("/proc/self/exe").expect("the test's own program");
        let status = fs::read_to_string("/proc/self/status").expect("the test's own status");
        let uid = parse_status(&status).expect("parses").uid;

        let pids = running(&executable.to_string_lossy(), uid).expect("/proc is listed");

        assert!(pids.contains(&std::process::id()), "{pids:?}");
    }

    #[test]
    fn the_same_program_under_another_account_is_not_the_row_that_was_marked() {
        let executable = fs::read_link("/proc/self/exe").expect("the test's own program");
        let status = fs::read_to_string("/proc/self/status").expect("the test's own status");
        let uid = parse_status(&status).expect("parses").uid;

        let pids = running(&executable.to_string_lossy(), uid.wrapping_add(7919))
            .expect("/proc is listed");

        assert!(
            !pids.contains(&std::process::id()),
            "nginx as root and nginx as www-data are two rows, and stopping the workers \
             must not stop the master someone did not mark: {pids:?}"
        );
    }

    #[test]
    fn a_pid_is_asked_again_whether_it_still_runs_the_program_and_the_account_marked() {
        let executable = fs::read_link("/proc/self/exe").expect("the test's own program");
        let executable = executable.to_string_lossy();
        let status = fs::read_to_string("/proc/self/status").expect("the test's own status");
        let uid = parse_status(&status).expect("parses").uid;
        let pid = std::process::id();

        assert!(still_running(pid, &executable, Some(uid)));
        assert!(still_running(pid, &executable, None));
        assert!(
            !still_running(pid, "/usr/sbin/nginx", Some(uid)),
            "a number that now belongs to another program is not the process that was marked"
        );
        assert!(!still_running(
            pid,
            &executable,
            Some(uid.wrapping_add(7919))
        ));
    }

    #[test]
    fn a_program_whose_file_was_deleted_under_it_is_still_the_program_the_row_names() {
        assert!(same_program("/tmp/.x/nc (deleted)", "/tmp/.x/nc"));
        assert!(same_program("/usr/sbin/nginx", "/usr/sbin/nginx"));
        assert!(!same_program("/usr/sbin/nginx-debug", "/usr/sbin/nginx"));
    }
}
