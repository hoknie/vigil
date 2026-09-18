use std::fs;
use std::path::{Path, PathBuf};

use vigil_config::Watch;
use vigil_files::{Watched, watch_list_in};

use super::{unwatch, watch};

const OLD_LAYOUT: &str =
    "state_dir: /var/lib/vigil\nfiles:\n  paths:\n    - \"/etc/hosts\"\nreporters: []\n";

const LIST: &str = "\
# The files watched for a changed content.
files:
  - /etc/ssh/sshd_config
devices:
  include: []
  exclude: []
";

fn temporary() -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-watched-{}-{}",
        std::process::id(),
        NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(directory.join("collectors")).expect("temp dir");
    directory
}

fn host(directory: &Path, files_block: &str) -> String {
    let configuration = directory.join("vigil.yaml");
    fs::write(
        &configuration,
        format!(
            "state_dir: /var/lib/vigil\ncollectors_path: {}\n",
            directory.join("collectors").display()
        ),
    )
    .expect("writes");
    fs::write(directory.join("collectors/files.yaml"), files_block).expect("writes");
    configuration.display().to_string()
}

fn watched_in(file: &Path) -> Vec<Watched> {
    watch_list_in(&fs::read_to_string(file).expect("readable"))
        .expect("the collector reads it")
        .files
}

#[test]
fn with_no_collectors_path_the_path_goes_into_the_configuration_as_it_always_did() {
    let directory = temporary();
    let configuration = directory.join("vigil.yaml");
    fs::write(&configuration, OLD_LAYOUT).expect("writes");
    let configuration = configuration.display().to_string();

    watch(&configuration, &Watch::of("/etc/sudoers", None)).expect("writes");
    let after = fs::read_to_string(&configuration).expect("readable");

    assert!(after.contains("    - \"/etc/sudoers\"\n"), "{after}");
    assert!(after.contains("reporters: []"), "{after}");
}

#[test]
fn a_path_named_in_the_configuration_is_taken_out_of_it_and_the_rest_read_back() {
    let directory = temporary();
    let configuration = directory.join("vigil.yaml");
    fs::write(&configuration, OLD_LAYOUT).expect("writes");
    let configuration = configuration.display().to_string();
    watch(&configuration, &Watch::of("/etc/sudoers", None)).expect("writes");

    let done = unwatch(&configuration, "/etc/hosts").expect("writes");

    assert_eq!(done.entries, 1, "{:?}", done.said);
    let after = fs::read_to_string(&configuration).expect("readable");
    assert!(!after.contains("/etc/hosts"), "{after}");
    assert!(after.contains("/etc/sudoers"), "{after}");
}

#[test]
fn a_watched_path_that_names_a_file_is_the_file_the_console_writes_into() {
    let directory = temporary();
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, LIST).expect("writes");
    let configuration = host(
        &directory,
        &format!(
            "files:\n  schedule: 300\n  watched_path: {}\n  max_file_size: 30mb\n",
            list.display()
        ),
    );

    let done = watch(&configuration, &Watch::of("/etc/ssh/*.conf", Some(8192))).expect("writes");

    assert_eq!(
        watched_in(&list),
        vec![
            Watched::of("/etc/ssh/sshd_config", None),
            Watched::of("/etc/ssh/*.conf", Some(8192)),
        ]
    );
    let after = fs::read_to_string(&list).expect("readable");
    assert!(
        after.starts_with("# The files watched"),
        "the comment an operator wrote stays: {after}"
    );
    assert!(
        done.said
            .iter()
            .any(|said| said.contains("next reading of the files")),
        "{:?}",
        done.said
    );
    assert!(
        fs::read_to_string(directory.join("collectors/files.yaml"))
            .expect("readable")
            .contains("max_file_size: 30mb"),
        "the block that names the list is not the file the list is written into"
    );
}

#[test]
fn a_watched_path_that_names_a_directory_takes_the_console_entries_in_a_file_of_their_own() {
    let directory = temporary();
    let lists = directory.join("watch_fs.d");
    fs::create_dir_all(&lists).expect("a directory of lists");
    fs::write(lists.join("10-base.yaml"), LIST).expect("writes");
    let configuration = host(
        &directory,
        &format!("files:\n  watched_path: {}\n", lists.display()),
    );

    watch(&configuration, &Watch::of("/etc/pam.d", None)).expect("writes");

    assert_eq!(
        watched_in(&lists.join("console.yaml")),
        vec![Watched::of("/etc/pam.d", None)]
    );
    assert_eq!(
        fs::read_to_string(lists.join("10-base.yaml")).expect("readable"),
        LIST,
        "a list the operator keeps is never rewritten by the console"
    );
}

#[test]
fn a_watch_list_that_is_not_there_yet_is_written_with_the_one_entry_asked_for() {
    let directory = temporary();
    let list = directory.join("watch_fs.yaml");
    let configuration = host(
        &directory,
        &format!("files:\n  watched_path: {}\n", list.display()),
    );

    watch(&configuration, &Watch::of("/etc/hosts", None)).expect("writes");

    assert_eq!(watched_in(&list), vec![Watched::of("/etc/hosts", None)]);
}

#[test]
fn a_path_stopped_leaves_the_watch_list_and_nothing_else_in_it() {
    let directory = temporary();
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, LIST).expect("writes");
    let configuration = host(
        &directory,
        &format!("files:\n  watched_path: {}\n", list.display()),
    );
    watch(&configuration, &Watch::of("/etc/hosts", None)).expect("writes");

    let done = unwatch(&configuration, "/etc/ssh/sshd_config").expect("writes");

    assert_eq!(done.entries, 1);
    assert_eq!(watched_in(&list), vec![Watched::of("/etc/hosts", None)]);
    assert!(
        fs::read_to_string(&list)
            .expect("readable")
            .contains("devices:\n  include: []"),
    );
}

#[test]
fn a_host_whose_collectors_name_no_files_block_is_told_how_to_get_one() {
    let directory = temporary();
    let configuration = host(&directory, "network:\n  schedule: 30\n");

    let refusal = watch(&configuration, &Watch::of("/etc/hosts", None))
        .expect_err("there is no list to write into");

    assert!(
        refusal.contains("vigild collector files enable"),
        "{refusal}"
    );
}

#[test]
fn a_files_block_that_names_no_watch_list_is_told_which_key_to_add() {
    let directory = temporary();
    let configuration = host(&directory, "files:\n  schedule: 300\n");

    let refusal = watch(&configuration, &Watch::of("/etc/hosts", None))
        .expect_err("there is no list to write into");

    assert!(refusal.contains("watched_path"), "{refusal}");
}

#[test]
fn a_files_block_switched_off_is_written_into_and_the_reader_is_told_nothing_is_watched_yet() {
    let directory = temporary();
    let list = directory.join("watch_fs.yaml");
    let configuration = host(
        &directory,
        &format!(
            "files:\n  enabled: false\n  watched_path: {}\n",
            list.display()
        ),
    );

    let done = watch(&configuration, &Watch::of("/etc/hosts", None)).expect("writes");

    assert!(
        done.said.iter().any(|said| said.contains("switched off")),
        "{:?}",
        done.said
    );
}

#[test]
fn a_files_block_that_still_names_its_paths_inside_is_edited_where_it_names_them() {
    let directory = temporary();
    let configuration = host(
        &directory,
        "files:\n  schedule: 300\n  paths:\n    - \"/etc/hosts\"\n  ceiling_bytes: 1048576\n",
    );

    watch(&configuration, &Watch::of("/etc/sudoers", Some(4096))).expect("writes");

    let after = fs::read_to_string(directory.join("collectors/files.yaml")).expect("readable");
    assert!(after.contains("  schedule: 300\n"), "{after}");
    assert!(after.contains("      ceiling_bytes: 4096\n"), "{after}");
}
