use serde::{Deserialize, Serialize};
use vigil_files::{CEILING_BYTES, WATCHED_BY_DEFAULT};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct WatchedFiles {
    pub paths: Vec<String>,
    pub ceiling_bytes: u64,
}

impl Default for WatchedFiles {
    fn default() -> Self {
        WatchedFiles {
            paths: WATCHED_BY_DEFAULT
                .iter()
                .map(|path| (*path).to_string())
                .collect(),
            ceiling_bytes: CEILING_BYTES,
        }
    }
}

impl WatchedFiles {
    pub fn check(&self) -> Result<(), String> {
        if self.ceiling_bytes == 0 {
            return Err(
                "files ceiling_bytes: 0 hashes nothing, and a file nobody hashes is a file \
                 nobody watches"
                    .to_string(),
            );
        }
        for path in &self.paths {
            if !path.starts_with('/') {
                return Err(format!(
                    "files paths: {path:?} is not an absolute path, and this agent reads no \
                     working directory of its own"
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
    fn a_file_that_says_nothing_watches_the_files_this_product_ships_a_list_of() {
        let written = WatchedFiles::default();

        assert!(written.paths.contains(&"/etc/ssh/sshd_config".to_string()));
        assert_eq!(written.ceiling_bytes, 1024 * 1024);
        assert!(written.check().is_ok());
    }

    #[test]
    fn the_list_is_of_files_and_never_of_trees_to_walk() {
        for path in &WatchedFiles::default().paths {
            assert!(
                !path.ends_with('/'),
                "{path} is a directory, and a directory in this list is a walk that hashes \
                 everything under it: the shortest path to an agent the operator turns off"
            );
        }
    }

    #[test]
    fn none_of_the_paths_this_product_ships_is_one_another_collector_already_reports_on() {
        for already in [
            "/etc/passwd",
            "/etc/shadow",
            "/etc/group",
            "/etc/sudoers",
            "/etc/crontab",
            "/etc/ld.so.preload",
        ] {
            assert!(
                !WatchedFiles::default().paths.contains(&already.to_string()),
                "{already} is read by another collector of this build, which reports what \
                 changed in it by name; hashing it here as well would put two findings on one \
                 edit"
            );
        }
    }

    #[test]
    fn a_relative_path_is_refused_at_the_door_and_not_at_the_first_reading() {
        let relative = WatchedFiles {
            paths: vec!["etc/hosts".into()],
            ..WatchedFiles::default()
        };

        assert!(relative.check().is_err());
        assert!(
            WatchedFiles {
                ceiling_bytes: 0,
                ..WatchedFiles::default()
            }
            .check()
            .is_err()
        );
    }

    #[test]
    fn a_host_that_names_no_path_at_all_is_a_file_this_product_reads_and_a_collector_that_says_so()
    {
        let none = WatchedFiles {
            paths: Vec::new(),
            ..WatchedFiles::default()
        };

        assert!(
            none.check().is_ok(),
            "naming no path is a decision an operator is allowed to make; what it must not be \
             is a reading that says every file is as it was, and the collector answers that \
             with Unavailable rather than with an empty snapshot"
        );
    }
}
