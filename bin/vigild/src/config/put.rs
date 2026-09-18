use std::path::Path;

use vigil_config::write;

pub const RESTART: &str = "systemctl try-restart vigild.service";

pub const READS_AT_START: &str = "the daemon reads its configuration at start";

pub fn put(path: &str, before: &str, after: &str, dry_run: bool) -> Result<String, String> {
    put_beside(path, Path::new(path), Some(before), after, dry_run)
}

pub fn put_beside(
    configuration: &str,
    file: &Path,
    before: Option<&str>,
    after: &str,
    dry_run: bool,
) -> Result<String, String> {
    if dry_run {
        println!("{after}");
        return Ok(format!(
            "nothing was written to {} (--dry-run)",
            file.display()
        ));
    }

    let written = write(file, after, true)?;
    super::load(configuration).map_err(|error| {
        let _ = match before {
            Some(before) => std::fs::write(file, before),
            None => std::fs::remove_file(file),
        };
        format!(
            "what this command wrote would not load, so {} was put back: {error}",
            file.display()
        )
    })?;

    Ok(match &written.previous {
        Some(previous) => format!(
            "wrote {} (0600), and what was there is kept as {}",
            written.path.display(),
            previous.display()
        ),
        None => format!("wrote {} (0600)", written.path.display()),
    })
}

pub fn restart_note() -> String {
    format!("{READS_AT_START}: {RESTART}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temporary(name: &str) -> std::path::PathBuf {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigil-put-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|since| since.as_nanos()
                    + NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed) as u128)
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&directory).expect("temp dir");
        directory.join(name)
    }

    #[test]
    fn what_would_not_load_is_never_left_on_the_disk_for_the_daemon_to_die_on() {
        let path = temporary("broken.yaml");
        let before = "retention_days: 90\n";
        std::fs::write(&path, before).expect("writes");
        let name = path.to_str().expect("utf-8");

        let refused = put(name, before, "retention_days: yesterday\n", false)
            .expect_err("must not be accepted");

        assert!(refused.contains("was put back"), "{refused}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            before,
            "the file an operator has to live with is the one that was there"
        );
    }

    #[test]
    fn a_file_of_collectors_that_would_not_load_is_taken_away_again_when_it_was_not_there_before() {
        let path = temporary("vigil.yaml");
        std::fs::write(&path, "collectors_path: collectors\n").expect("writes");
        let file = path.with_file_name("collectors").join("users.yaml");

        let refused = put_beside(
            path.to_str().expect("utf-8"),
            &file,
            None,
            "users:\n  schedule: never\n",
            false,
        )
        .expect_err("must not be accepted");

        assert!(refused.contains("users.yaml"), "{refused}");
        assert!(
            !file.exists(),
            "a file this command made and the daemon would refuse is not left for it to die on"
        );
    }

    #[test]
    fn a_dry_run_writes_nothing_at_all_and_says_so() {
        let path = temporary("untouched.yaml");
        std::fs::write(&path, "retention_days: 90\n").expect("writes");
        let name = path.to_str().expect("utf-8");

        let said = put(name, "retention_days: 90\n", "retention_days: 5\n", true).expect("says");

        assert!(said.contains("--dry-run"), "{said}");
        assert_eq!(
            std::fs::read_to_string(&path).expect("readable"),
            "retention_days: 90\n"
        );
    }

    #[test]
    fn every_command_that_edits_the_file_says_the_same_thing_about_when_it_is_read() {
        assert!(restart_note().contains(RESTART));
        assert!(restart_note().contains("reads its configuration at start"));
    }
}
