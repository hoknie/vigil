pub const GRAIN_PERCENT: u64 = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filesystem {
    pub mount: String,
    pub device: String,
    pub kind: String,
    pub read_only: bool,
    pub total_bytes: u64,
    pub free_percent_step: Option<u64>,
    pub free_inodes_percent_step: Option<u64>,
}

pub fn free_percent_step(available: u64, total: u64) -> Option<u64> {
    if total == 0 || available > total {
        return None;
    }

    let percent = available.checked_mul(100)? / total;

    Some(percent / GRAIN_PERCENT * GRAIN_PERCENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_space_is_recorded_as_a_step_and_never_as_the_bytes_that_are_free() {
        assert_eq!(free_percent_step(37, 100), Some(35));
        assert_eq!(free_percent_step(39, 100), Some(35));
        assert_eq!(free_percent_step(40, 100), Some(40));

        assert_eq!(
            free_percent_step(41_000_000_000, 100_000_000_000),
            free_percent_step(40_999_999_000, 100_000_000_000),
            "the bytes free on a filesystem anything is writing to differ between any two \
             readings, so a reading carrying them differs from the one before it every time"
        );
    }

    #[test]
    fn a_filesystem_that_is_full_and_one_that_is_nearly_full_are_two_different_steps() {
        assert_eq!(free_percent_step(0, 100), Some(0));
        assert_eq!(free_percent_step(4, 100), Some(0));
        assert_eq!(free_percent_step(5, 100), Some(5));
        assert_eq!(free_percent_step(100, 100), Some(100));
    }

    #[test]
    fn a_filesystem_that_counts_nothing_is_not_a_filesystem_with_nothing_left() {
        assert_eq!(
            free_percent_step(0, 0),
            None,
            "btrfs and xfs report no inode total at all, and reading that as zero free would \
             be a host out of inodes on every reading"
        );
        assert_eq!(free_percent_step(7, 3), None);
    }

    #[test]
    fn a_filesystem_the_size_of_a_fleet_is_stepped_without_overflowing() {
        let huge = u64::MAX / 100;

        assert_eq!(free_percent_step(huge / 2, huge), Some(50));
        assert_eq!(free_percent_step(u64::MAX, u64::MAX), None);
    }
}
