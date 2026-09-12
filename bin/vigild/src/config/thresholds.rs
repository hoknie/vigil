use serde::{Deserialize, Serialize};
use vigil_rules::{CLOCK_SKEW_SECONDS, DISK_FREE_PERCENT, INODE_FREE_PERCENT, ResourceLimits};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct ResourceThresholds {
    pub clock_skew_seconds: u32,
    pub disk_free_percent: u32,
    pub inode_free_percent: u32,
}

impl Default for ResourceThresholds {
    fn default() -> Self {
        ResourceThresholds {
            clock_skew_seconds: CLOCK_SKEW_SECONDS,
            disk_free_percent: DISK_FREE_PERCENT,
            inode_free_percent: INODE_FREE_PERCENT,
        }
    }
}

impl ResourceThresholds {
    pub fn limits(&self) -> ResourceLimits {
        ResourceLimits {
            clock_skew_seconds: i64::from(self.clock_skew_seconds),
            disk_free_percent: u64::from(self.disk_free_percent),
            inode_free_percent: u64::from(self.inode_free_percent),
        }
    }

    pub fn check(&self) -> Result<(), String> {
        if self.clock_skew_seconds == 0 {
            return Err(
                "resources clock_skew_seconds: 0 reports every reading as a clock that moved"
                    .to_string(),
            );
        }
        for (named, percent) in [
            ("disk_free_percent", self.disk_free_percent),
            ("inode_free_percent", self.inode_free_percent),
        ] {
            if percent > 100 {
                return Err(format!(
                    "resources {named}: {percent} is more than a filesystem can have free"
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_says_nothing_about_the_limits_runs_on_the_ones_the_rules_declare() {
        let written = ResourceThresholds::default();

        assert_eq!(written.limits(), ResourceLimits::default());
        assert_eq!(written.clock_skew_seconds, 300);
    }

    #[test]
    fn a_limit_a_file_names_is_the_one_the_rules_are_built_with() {
        let named = ResourceThresholds {
            clock_skew_seconds: 30,
            disk_free_percent: 25,
            inode_free_percent: 5,
        };

        assert_eq!(named.limits().clock_skew_seconds, 30);
        assert_eq!(named.limits().disk_free_percent, 25);
        assert_eq!(named.limits().inode_free_percent, 5);
    }

    #[test]
    fn a_limit_that_would_report_every_reading_is_refused_at_the_door_and_not_at_the_first_tick() {
        assert!(
            ResourceThresholds {
                clock_skew_seconds: 0,
                ..ResourceThresholds::default()
            }
            .check()
            .is_err()
        );
        assert!(
            ResourceThresholds {
                disk_free_percent: 101,
                ..ResourceThresholds::default()
            }
            .check()
            .is_err()
        );
        assert!(ResourceThresholds::default().check().is_ok());
    }
}
