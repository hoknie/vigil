use std::path::{Path, PathBuf};

use crate::collector::apart::switch;
use crate::collector::run::Options;
use crate::config::load;

const POINTING: &str = "collectors_path: collectors\n";

const USERS: &str = "\
users:
  # How often collector reads, in seconds.
  schedule: 300

  accounts:
    from_the_console: false
";

struct Bench {
    directory: PathBuf,
}

impl Bench {
    fn new(named: &str) -> Bench {
        static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "vigild-collector-apart-{named}-{}-{}",
            std::process::id(),
            NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(directory.join("collectors")).expect("a directory");
        std::fs::write(directory.join("vigil.yaml"), POINTING).expect("write");
        Bench { directory }
    }

    fn with(self, file: &str, text: &str) -> Bench {
        std::fs::write(self.directory.join("collectors").join(file), text).expect("write");
        self
    }

    fn options(&self, name: &str, dry_run: bool) -> Options {
        Options {
            name: name.to_string(),
            path: self.directory.join("vigil.yaml").display().to_string(),
            dry_run,
        }
    }

    fn at(&self) -> PathBuf {
        self.directory.join("collectors")
    }

    fn file(&self, name: &str) -> String {
        std::fs::read_to_string(self.at().join(name)).expect("readable")
    }

    fn switch(&self, name: &str, enabled: bool) -> Result<String, String> {
        switch(&self.options(name, false), name, &self.at(), enabled)
    }

    fn on(&self) -> Vec<String> {
        load(&self.options("", false).path)
            .expect("what the command left loads")
            .collectors
            .expect("the blocks name what runs")
    }
}

impl Drop for Bench {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn switching_a_collector_off_writes_one_line_into_its_own_block_and_nothing_else() {
    let bench = Bench::new("off").with("users.yaml", USERS);

    let said = bench.switch("users", false).expect("switches");

    assert!(said.contains("users.yaml"), "{said}");
    assert_eq!(
        bench.file("users.yaml"),
        USERS.replacen("users:\n", "users:\n  enabled: false\n", 1),
        "the comments and the verb beside it are the operator's and stay"
    );
    assert!(bench.on().is_empty());
}

#[test]
fn switching_it_back_on_turns_the_same_line_and_the_daemon_reads_it_as_running() {
    let bench = Bench::new("on-again").with("users.yaml", USERS);
    bench.switch("users", false).expect("switches off");

    bench.switch("users", true).expect("switches on");

    assert_eq!(
        bench.file("users.yaml"),
        USERS.replacen("users:\n", "users:\n  enabled: true\n", 1)
    );
    assert_eq!(bench.on(), ["users"]);
}

#[test]
fn a_collector_with_no_block_is_given_a_file_of_its_own_with_the_period_it_declares() {
    let bench = Bench::new("new-block");

    bench.switch("launches", true).expect("switches on");

    let every_seconds = crate::modules::every_seconds_of("launches").expect("known");
    assert_eq!(
        bench.file("launches.yaml"),
        format!("launches:\n  schedule: {every_seconds}\n")
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(bench.at().join("launches.yaml"))
            .expect("stat")
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "mode was {:o}", mode & 0o777);
    }
    assert_eq!(bench.on(), ["launches"]);
}

#[test]
fn a_collector_already_on_or_already_off_is_not_written_again() {
    let bench = Bench::new("already").with("users.yaml", USERS);

    let on = bench.switch("users", true).expect("says");
    let off = bench.switch("launches", false).expect("says");

    assert!(on.contains("already on"), "{on}");
    assert!(
        off.contains("already off") && off.contains("no block"),
        "{off}"
    );
    assert_eq!(bench.file("users.yaml"), USERS);
    assert!(!bench.at().join("launches.yaml").exists());
}

#[test]
fn a_dry_run_writes_nothing_into_the_collectors_directory() {
    let bench = Bench::new("dry").with("users.yaml", USERS);

    let said = switch(&bench.options("users", true), "users", &bench.at(), false).expect("says");
    switch(&bench.options("files", true), "files", &bench.at(), true).expect("says");

    assert!(said.contains("--dry-run"), "{said}");
    assert_eq!(bench.file("users.yaml"), USERS);
    assert!(!bench.at().join("files.yaml").exists());
}

#[test]
fn a_collectors_path_that_names_one_file_is_given_the_new_block_at_its_end() {
    let bench = Bench::new("one-file");
    let file = bench.directory.join("collectors.yaml");
    std::fs::write(
        bench.directory.join("vigil.yaml"),
        "collectors_path: collectors.yaml\n",
    )
    .expect("write");
    std::fs::write(&file, USERS).expect("write");

    switch(&bench.options("launches", false), "launches", &file, true).expect("switches on");

    let text = std::fs::read_to_string(&file).expect("readable");
    assert!(text.starts_with(USERS.trim_end()), "{text}");
    assert_eq!(bench.on(), ["users", "launches"]);
}

#[test]
fn the_configuration_says_where_the_collectors_are_and_a_file_without_the_key_has_none() {
    let bench = Bench::new("where");
    let options = bench.options("users", false);

    assert_eq!(
        crate::collector::apart::collectors_at(&options, POINTING),
        Ok(Some(bench.at()))
    );
    assert_eq!(
        crate::collector::apart::collectors_at(&options, "collectors: [users]\n"),
        Ok(None),
        "a file of the former layout is edited as it always was"
    );
    assert!(Path::new(&options.path).is_absolute());
}
