pub const CLOCK_SKEW_SECONDS: u32 = 300;

pub const DISK_FREE_PERCENT: u32 = 10;

pub const INODE_FREE_PERCENT: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceLimits {
    pub clock_skew_seconds: i64,
    pub disk_free_percent: u64,
    pub inode_free_percent: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            clock_skew_seconds: i64::from(CLOCK_SKEW_SECONDS),
            disk_free_percent: u64::from(DISK_FREE_PERCENT),
            inode_free_percent: u64::from(INODE_FREE_PERCENT),
        }
    }
}

impl ResourceLimits {
    pub fn below_the_disk_limit(&self, step: Option<u64>) -> bool {
        matches!(step, Some(step) if step < self.disk_free_percent)
    }

    pub fn below_the_inode_limit(&self, step: Option<u64>) -> bool {
        matches!(step, Some(step) if step < self.inode_free_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filesystem_that_says_nothing_about_its_inodes_is_never_one_that_has_run_out() {
        let limits = ResourceLimits::default();

        assert!(!limits.below_the_inode_limit(None));
        assert!(!limits.below_the_disk_limit(None));
    }

    #[test]
    fn a_step_is_the_room_a_filesystem_has_at_least_so_the_limit_bites_below_it_and_not_at_it() {
        let limits = ResourceLimits::default();

        assert!(limits.below_the_disk_limit(Some(5)));
        assert!(!limits.below_the_disk_limit(Some(10)));
        assert!(
            !limits.below_the_disk_limit(Some(15)),
            "a step of 10 is a filesystem with somewhere between 10 and 15 percent free, so a \
             limit of 10 has not been crossed yet and saying it has is a false alarm"
        );
        assert!(limits.below_the_disk_limit(Some(0)));
    }

    #[test]
    fn the_limits_a_host_runs_on_when_its_file_says_nothing_are_the_ones_written_here() {
        let limits = ResourceLimits::default();

        assert_eq!(limits.clock_skew_seconds, 300);
        assert_eq!(limits.disk_free_percent, 10);
        assert_eq!(limits.inode_free_percent, 10);
    }
}
