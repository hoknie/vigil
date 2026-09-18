use serde::{Deserialize, Serialize};

use crate::helpers::check_mask;

const CEILING_BYTES: &str = "ceiling_bytes";

const MAX_FILE_SIZE: &str = "max_file_size";

pub const LONGEST_PATH: usize = 4096;

pub const MOST_HASHED_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Watched {
    Named(String),
    Held { path: String, ceiling_bytes: u64 },
}

impl Watched {
    pub fn of(path: impl Into<String>, ceiling_bytes: Option<u64>) -> Watched {
        let path = path.into();

        match ceiling_bytes {
            Some(ceiling_bytes) => Watched::Held {
                path,
                ceiling_bytes,
            },
            None => Watched::Named(path),
        }
    }

    pub fn path(&self) -> &str {
        match self {
            Watched::Named(path) | Watched::Held { path, .. } => path,
        }
    }

    pub fn ceiling_bytes(&self) -> Option<u64> {
        match self {
            Watched::Named(_) => None,
            Watched::Held { ceiling_bytes, .. } => Some(*ceiling_bytes),
        }
    }

    pub fn hashed_to(&self, fallback: u64) -> u64 {
        self.ceiling_bytes().unwrap_or(fallback)
    }

    pub fn check(&self) -> Result<(), String> {
        self.readable()?;
        let path = self.path();
        if path.len() > 1 && path.ends_with('/') {
            return Err(format!(
                "{path:?} is a directory, and a directory in this list is a walk that hashes \
                 everything under it: name the file itself"
            ));
        }

        self.hashed(CEILING_BYTES)
    }

    pub fn check_listed(&self) -> Result<(), String> {
        self.readable()?;
        check_mask(self.path())?;

        self.hashed(MAX_FILE_SIZE)
    }

    fn readable(&self) -> Result<(), String> {
        let path = self.path();
        if path.trim().is_empty() {
            return Err(
                "a watched path with nothing in it watches nothing, and the agent would read \
                 the list back with one entry it can never stat"
                    .to_string(),
            );
        }
        if !path.starts_with('/') {
            return Err(format!(
                "{path:?} is not an absolute path, and this agent reads no working directory of \
                 its own"
            ));
        }
        if path.chars().any(char::is_control) {
            return Err(format!(
                "{path:?} holds a character the configuration file cannot carry, so what was \
                 written would not be the path read back"
            ));
        }
        if path.len() > LONGEST_PATH {
            return Err(format!(
                "a path of {} characters is longer than any path this kernel opens, so nothing \
                 would ever be read from it",
                path.len()
            ));
        }
        Ok(())
    }

    fn hashed(&self, key: &str) -> Result<(), String> {
        match self.ceiling_bytes() {
            None => Ok(()),
            Some(0) => Err(format!(
                "{key}: 0 for {:?} hashes nothing, and a file nobody hashes is a file \
                 whose content nobody watches",
                self.path()
            )),
            Some(ceiling) if ceiling > MOST_HASHED_BYTES => Err(format!(
                "{key}: {ceiling} for {:?} is over the {MOST_HASHED_BYTES} this agent \
                 hashes on one pass, and a reading that takes longer than the pass it belongs \
                 to is the agent an operator turns off first",
                self.path()
            )),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_written_as_a_word_and_one_written_with_its_own_ceiling_are_both_watched() {
        let plain: Watched = serde_json::from_str("\"/etc/hosts\"").expect("reads");
        let held: Watched =
            serde_json::from_str("{\"path\":\"/etc/hosts\",\"ceiling_bytes\":4096}")
                .expect("reads");

        assert_eq!(plain.path(), "/etc/hosts");
        assert_eq!(plain.ceiling_bytes(), None);
        assert_eq!(held.ceiling_bytes(), Some(4096));
        assert_eq!(
            plain.hashed_to(1024),
            1024,
            "a path that names no ceiling of its own is hashed to the one the block names, so a \
             file written the way every file was written before this build behaves as it did"
        );
        assert_eq!(held.hashed_to(1024), 4096);
    }

    #[test]
    fn a_path_this_agent_could_never_read_back_is_refused_in_words_that_say_which_part_is_wrong() {
        for (watched, said) in [
            (Watched::of("etc/hosts", None), "absolute"),
            (Watched::of("/etc/pam.d/", None), "directory"),
            (Watched::of("  ", None), "watches nothing"),
            (Watched::of("/etc/ho\nsts", None), "cannot carry"),
            (Watched::of("/etc/hosts", Some(0)), "hashes nothing"),
            (
                Watched::of("/etc/hosts", Some(MOST_HASHED_BYTES + 1)),
                "over the",
            ),
        ] {
            let refusal = watched.check().expect_err("must not be accepted");
            assert!(refusal.contains(said), "{watched:?}: {refusal}");
        }
    }

    #[test]
    fn a_path_the_agent_reads_every_pass_is_accepted_however_it_was_written() {
        for watched in [
            Watched::of("/etc/hosts", None),
            Watched::of("/etc/ssh/sshd_config", Some(1024 * 1024)),
            Watched::of("/", None),
            Watched::of("/etc/a file with spaces", None),
        ] {
            assert_eq!(watched.check(), Ok(()), "{watched:?}");
        }
    }

    #[test]
    fn a_path_that_names_no_ceiling_goes_back_into_the_file_as_the_word_it_came_out_of() {
        let plain = serde_json::to_string(&Watched::of("/etc/hosts", None)).expect("writes");
        let held = serde_json::to_string(&Watched::of("/etc/hosts", Some(4096))).expect("writes");

        assert_eq!(plain, "\"/etc/hosts\"");
        assert!(held.contains("ceiling_bytes"), "{held}");
        assert!(
            !plain.contains("ceiling_bytes"),
            "writing a ceiling against every path would rewrite a file the operator keeps in \
             version control for a change nobody asked for"
        );
    }

    #[test]
    fn a_directory_or_a_mask_in_a_watch_list_is_a_walk_it_asks_for_and_not_a_mistake() {
        for watched in [
            Watched::of("/etc/pam.d", None),
            Watched::of("/etc/pam.d/", None),
            Watched::of("/etc/ssh/*.conf", Some(4096)),
            Watched::of("/etc/[a-z]*.d/sshd", None),
        ] {
            assert_eq!(watched.check_listed(), Ok(()), "{watched:?}");
        }
    }

    #[test]
    fn a_watch_list_refuses_what_the_old_list_refused_and_says_it_in_the_key_it_is_written_in() {
        for (watched, said) in [
            (Watched::of("etc/hosts", None), "absolute"),
            (Watched::of("/etc/ho\nsts", None), "cannot carry"),
            (Watched::of("/etc/hosts", Some(0)), "max_file_size: 0"),
            (Watched::of("/etc/[ab", None), "never closed"),
            (
                Watched::of("/etc/hosts", Some(MOST_HASHED_BYTES + 1)),
                "max_file_size",
            ),
        ] {
            let refusal = watched.check_listed().expect_err("must not be accepted");
            assert!(refusal.contains(said), "{watched:?}: {refusal}");
        }
    }
}
