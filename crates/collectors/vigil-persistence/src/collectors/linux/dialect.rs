use std::path::Path;

use super::DEBIAN_VERSION;

const LEFT_BY_A_PACKAGE_MANAGER: &[&str] = &[".rpmsave", ".rpmorig", ".rpmnew"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CronDialect {
    Debian,
    Cronie,
}

impl CronDialect {
    pub(super) fn of_this_host() -> CronDialect {
        match Path::new(DEBIAN_VERSION).exists() {
            true => CronDialect::Debian,
            false => CronDialect::Cronie,
        }
    }

    pub(super) fn runs_from_cron_d(self, name: &str) -> bool {
        match self {
            CronDialect::Debian => {
                !name.is_empty()
                    && name.chars().all(|character| {
                        character.is_ascii_alphanumeric() || "_-".contains(character)
                    })
            }
            CronDialect::Cronie => {
                !name.is_empty()
                    && !name.starts_with('.')
                    && !name.starts_with('#')
                    && !name.ends_with('~')
                    && !LEFT_BY_A_PACKAGE_MANAGER
                        .iter()
                        .any(|suffix| name.ends_with(suffix))
            }
        }
    }
}
