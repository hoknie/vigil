pub fn runs_as(pid: u32, uid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/status"))
        .ok()
        .and_then(|status| uid_in(&status))
        .is_some_and(|found| found == uid)
}

fn uid_in(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|real| real.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_account_a_process_runs_as_is_its_real_uid_from_the_status_file() {
        let status = "Name:\tsshd\nUid:\t1000\t1000\t1000\t1000\nGid:\t1000\t1000\t1000\t1000\n";

        assert_eq!(uid_in(status), Some(1000));
        assert_eq!(
            uid_in("Name:\tsshd\n"),
            None,
            "a status with no Uid line is not a process of uid 0"
        );
    }
}
