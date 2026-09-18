use serde::Deserialize;

use crate::types::Watching;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct Held {
    files: Watching,
}

pub fn watching_in(text: &str) -> Result<Watching, String> {
    let held: Held = serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    held.files.check()?;

    Ok(held.files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Watched;

    #[test]
    fn a_file_that_names_no_files_block_watches_the_list_this_product_ships() {
        let held = watching_in("state_dir: /var/lib/vigil\n").expect("reads");

        assert_eq!(held.hashed().len(), WATCHED.len());
        for (_, ceiling_bytes) in held.hashed() {
            assert_eq!(ceiling_bytes, crate::types::CEILING_BYTES);
        }
    }

    const WATCHED: &[&str] = crate::types::WATCHED_BY_DEFAULT;

    #[test]
    fn the_block_a_file_names_is_read_back_entry_by_entry_with_every_other_key_left_alone() {
        let held = watching_in(
            "state_dir: /var/lib/vigil\nfiles:\n  paths:\n    - \"/etc/hosts\"\n    - path: \"/etc/sudoers\"\n      ceiling_bytes: 4096\n  ceiling_bytes: 2048\nreporters: []\n",
        )
        .expect("reads");

        assert_eq!(
            held.paths,
            Some(vec![
                Watched::of("/etc/hosts", None),
                Watched::of("/etc/sudoers", Some(4096)),
            ])
        );
        assert_eq!(
            held.hashed(),
            vec![
                ("/etc/hosts".to_string(), 2048),
                ("/etc/sudoers".to_string(), 4096),
            ]
        );
    }

    #[test]
    fn a_block_the_daemon_would_refuse_at_start_up_is_refused_here_first() {
        let refusal = watching_in("files:\n  paths:\n    - \"etc/hosts\"\n")
            .expect_err("a path this agent cannot stat is a path nobody meant to write");

        assert!(refusal.contains("absolute"), "{refusal}");
    }

    #[test]
    fn a_key_this_build_does_not_know_is_refused_rather_than_quietly_left_out() {
        assert!(
            watching_in("files:\n  path:\n    - \"/etc/hosts\"\n").is_err(),
            "a misspelled key leaves the shipped list watched and nothing says so"
        );
    }

    #[test]
    fn a_file_that_is_not_yaml_at_all_says_so_rather_than_reading_as_a_host_watching_nothing() {
        assert!(watching_in("files:\n  - - - :\n").is_err());
    }
}
