use std::fs;
use std::path::PathBuf;

use super::records_arguments;

fn workspace(named: &str) -> PathBuf {
    let at = std::env::temp_dir().join(format!(
        "vigil-launches-spool-{}-{named}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&at);
    fs::create_dir_all(at.join("collectors")).expect("a directory");
    at
}

#[test]
fn arguments_are_recorded_only_when_the_launches_block_says_so() {
    let at = workspace("said");
    let configuration = at.join("vigil.yaml");
    fs::write(&configuration, "collectors_path: collectors\n").expect("write");
    fs::write(
        at.join("collectors/launches.yaml"),
        "launches:\n  schedule: 15\n  record_arguments: true\n",
    )
    .expect("write");

    assert_eq!(records_arguments(&configuration), Ok(true));

    fs::write(
        at.join("collectors/launches.yaml"),
        "launches:\n  schedule: 15\n",
    )
    .expect("write");
    assert_eq!(records_arguments(&configuration), Ok(false));
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_block_switched_off_records_no_arguments_whatever_else_it_says() {
    let at = workspace("off");
    let configuration = at.join("vigil.yaml");
    fs::write(&configuration, "collectors_path: collectors\n").expect("write");
    fs::write(
        at.join("collectors/launches.yaml"),
        "launches:\n  enabled: false\n  record_arguments: true\n",
    )
    .expect("write");

    assert_eq!(records_arguments(&configuration), Ok(false));
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_configuration_that_names_its_collectors_itself_is_read_the_same_way() {
    let at = workspace("inline");
    let configuration = at.join("vigil.yaml");
    fs::write(
        &configuration,
        "state_dir: /usr/local/var/lib/vigil\nlaunches:\n  record_arguments: true\n",
    )
    .expect("write");

    assert_eq!(records_arguments(&configuration), Ok(true));
    let _ = fs::remove_dir_all(&at);
}

#[test]
fn a_configuration_that_cannot_be_read_is_an_error_the_caller_turns_into_no_arguments() {
    let at = workspace("missing");

    assert!(records_arguments(&at.join("vigil.yaml")).is_err());

    fs::write(
        at.join("vigil.yaml"),
        "launches:\n  record_arguments: sometimes\n",
    )
    .expect("write");
    assert!(records_arguments(&at.join("vigil.yaml")).is_err());
    let _ = fs::remove_dir_all(&at);
}
