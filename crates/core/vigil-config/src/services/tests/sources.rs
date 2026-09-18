use std::fs;
use std::path::{Path, PathBuf};

use crate::{CONSOLE_FILE, gathered, sources, suppressions_path, written_to};

fn temporary(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-sources-{name}-{}-{}",
        std::process::id(),
        NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("temp dir");
    directory
}

const ONE: &str = "suppressions:\n  - finding_key: \"user|group|docker\"\n    reason: ours\n";

const TWO: &str = "\
suppressions:
  - finding_key_prefix: \"port.listen|tcp|10.0.0.5:\"
    reason: the staging network
  - kind: port.listen.removed
    reason: nobody minds a service stopping here
";

#[test]
fn every_file_of_the_directory_is_read_and_says_where_each_entry_came_from() {
    let directory = temporary("every");
    fs::write(directory.join("10-deploy.yaml"), TWO).expect("writes");
    fs::write(directory.join(CONSOLE_FILE), ONE).expect("writes");

    let read = gathered(&directory).expect("reads");

    assert_eq!(read.len(), 2);
    assert_eq!(read[0].path, directory.join("10-deploy.yaml"));
    assert_eq!(read[0].suppressions.len(), 2);
    assert_eq!(read[1].path, directory.join(CONSOLE_FILE));
    assert_eq!(read[1].suppressions[0].reason, "ours");
}

#[test]
fn one_broken_file_is_named_rather_than_skipped_or_read_as_empty() {
    let directory = temporary("broken");
    fs::write(directory.join(CONSOLE_FILE), ONE).expect("writes");
    fs::write(
        directory.join("20-typo.yaml"),
        "suppressions:\n  - finding_key: \"a|b\"\n    raeson: typo\n",
    )
    .expect("writes");

    let refused = gathered(&directory).expect_err("must not be accepted");

    assert!(refused.contains("20-typo.yaml"), "{refused}");
    assert!(
        refused.contains("raeson"),
        "an entry skipped for a typo is a finding the operator believes is silenced: {refused}"
    );
}

#[test]
fn the_configuration_and_the_files_it_points_at_are_read_together_configuration_first() {
    let directory = temporary("together");
    let configuration = directory.join("vigil.yaml");
    let text = format!("{ONE}suppressions_path: silences\n");
    fs::create_dir_all(directory.join("silences")).expect("a directory");
    fs::write(directory.join("silences").join("deploy.yaml"), TWO).expect("writes");

    let read = sources(&configuration, &text).expect("reads");

    assert_eq!(
        read.iter()
            .map(|source| source.path.clone())
            .collect::<Vec<_>>(),
        vec![
            configuration.clone(),
            directory.join("silences").join("deploy.yaml")
        ]
    );
    assert_eq!(read[0].suppressions.len(), 1);
    assert_eq!(read[1].suppressions.len(), 2);
}

#[test]
fn a_configuration_that_points_nowhere_holds_what_it_says_itself_and_nothing_more() {
    let configuration = Path::new("/nonexistent/vigil/vigil.yaml");

    let read = sources(configuration, ONE).expect("reads");

    assert_eq!(read.len(), 1);
    assert_eq!(suppressions_path(configuration, ONE), Ok(None));
}

#[test]
fn what_the_console_writes_goes_into_a_file_of_its_own_and_not_over_one_somebody_wrote() {
    let directory = temporary("written");

    assert_eq!(
        written_to(&directory, None),
        Ok(directory.join(CONSOLE_FILE))
    );
    assert_eq!(
        written_to(&directory.join("not-made-yet"), None),
        Ok(directory.join("not-made-yet").join(CONSOLE_FILE)),
        "a directory the configuration names and nobody made yet is made by the first write"
    );
}

#[test]
fn a_file_asked_for_by_name_lands_in_the_directory_and_nowhere_else() {
    let directory = temporary("named");

    assert_eq!(
        written_to(&directory, Some("deploy")),
        Ok(directory.join("deploy.yaml"))
    );
    assert_eq!(
        written_to(&directory, Some("deploy.yml")),
        Ok(directory.join("deploy.yml"))
    );
    for refused in [
        "../vigil.yaml",
        "/etc/passwd",
        ".hidden",
        "notes.txt",
        "a/b.yaml",
    ] {
        assert!(written_to(&directory, Some(refused)).is_err(), "{refused}");
    }
}

#[test]
fn a_path_that_names_one_file_is_written_to_as_that_file_and_has_no_room_for_another() {
    let directory = temporary("single");
    let file = directory.join("silences.yaml");

    assert_eq!(written_to(&file, None), Ok(file.clone()));
    assert!(written_to(&file, Some("deploy")).is_err());
}
