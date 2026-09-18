use std::fs;
use std::path::{Path, PathBuf};

use super::lists::Lists;
use crate::types::Watched;

fn temporary(name: &str) -> PathBuf {
    static NAMES_GIVEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "vigil-files-lists-{name}-{}-{}",
        std::process::id(),
        NAMES_GIVEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).expect("temp dir");
    directory
}

fn paths(lists: &mut Lists, path: &Path) -> Vec<String> {
    lists
        .gather(path)
        .entries
        .iter()
        .map(|watched| watched.path().to_string())
        .collect()
}

#[test]
fn a_list_edited_while_the_agent_runs_is_what_the_next_reading_watches() {
    let directory = temporary("edited");
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, "files:\n  - /etc/hosts\n").expect("writes");
    let mut lists = Lists::default();

    assert_eq!(paths(&mut lists, &list), vec!["/etc/hosts"]);
    fs::write(&list, "files:\n  - /etc/hosts\n  - /etc/pam.d\n").expect("writes");

    assert_eq!(
        paths(&mut lists, &list),
        vec!["/etc/hosts", "/etc/pam.d"],
        "an edit applies on the next reading, with no restart and nothing asked of the daemon"
    );
}

#[test]
fn a_list_that_did_not_change_is_not_parsed_again() {
    let directory = temporary("unchanged");
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, "files:\n  - /etc/hosts\n").expect("writes");
    let mut lists = Lists::default();

    for _ in 0..5 {
        lists.gather(&list);
    }

    assert_eq!(
        lists.parsed(),
        1,
        "a stat says the list is the one already read, and a reading every five minutes of a \
         list nobody touched is parsing nobody asked for"
    );
}

#[test]
fn a_list_broken_by_an_edit_keeps_what_it_named_before_and_says_which_file_broke() {
    let directory = temporary("broken");
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, "files:\n  - /etc/hosts\n").expect("writes");
    let mut lists = Lists::default();
    lists.gather(&list);

    fs::write(&list, "files:\n  - etc/hosts-without-a-slash\n").expect("writes");
    let gathered = lists.gather(&list);

    assert_eq!(gathered.entries, vec![Watched::of("/etc/hosts", None)]);
    assert!(gathered.nothing.is_none());
    assert_eq!(gathered.complaints.len(), 1, "{:?}", gathered.complaints);
    assert!(
        gathered.complaints[0].contains(&list.display().to_string())
            && gathered.complaints[0].contains("watched until it reads again"),
        "{}",
        gathered.complaints[0]
    );

    fs::write(&list, "files:\n  - /etc/hosts\n  - /etc/login.defs\n").expect("writes");
    let mended = lists.gather(&list);
    assert!(mended.complaints.is_empty(), "{:?}", mended.complaints);
    assert_eq!(mended.entries.len(), 2);
}

#[test]
fn a_list_broken_from_its_first_reading_is_never_read_as_a_host_that_watches_nothing() {
    let directory = temporary("never");
    let list = directory.join("watch_fs.yaml");
    fs::write(&list, "files: [\n").expect("writes");

    let gathered = Lists::default().gather(&list);

    let said = gathered
        .nothing
        .expect("nothing is watched, and that is said");
    assert!(said.contains(&list.display().to_string()), "{said}");
    assert!(said.contains("does not read"), "{said}");
}

#[test]
fn a_list_that_is_not_there_is_said_plainly() {
    let directory = temporary("absent");
    let list = directory.join("watch_fs.yaml");

    let said = Lists::default()
        .gather(&list)
        .nothing
        .expect("a missing list watches nothing");

    assert!(said.contains("is not there"), "{said}");
    assert!(said.contains("watch_fs.yaml"), "{said}");
}

#[test]
fn a_directory_of_lists_watches_every_list_in_it_and_a_path_named_twice_once() {
    let directory = temporary("many");
    fs::write(
        directory.join("10-base.yaml"),
        "files:\n  - /etc/hosts\ndevices:\n  exclude: [nfs]\n",
    )
    .expect("writes");
    fs::write(
        directory.join("console.yaml"),
        "files:\n  - path: /etc/hosts\n    max_file_size: 4kb\n  - /etc/pam.d\ndevices:\n  exclude: [cifs]\n",
    )
    .expect("writes");

    let gathered = Lists::default().gather(&directory);

    assert_eq!(
        gathered.entries,
        vec![
            Watched::of("/etc/hosts", None),
            Watched::of("/etc/pam.d", None)
        ],
        "the list read first names the path first, and the order is the order of the names"
    );
    assert_eq!(gathered.devices.exclude, vec!["nfs", "cifs"]);
}

#[test]
fn a_directory_with_no_list_in_it_and_a_list_with_no_path_in_it_both_say_so() {
    let directory = temporary("empty");
    let empty_directory = Lists::default()
        .gather(&directory)
        .nothing
        .expect("says so");
    fs::write(directory.join("console.yaml"), "files: []\n").expect("writes");
    let empty_list = Lists::default()
        .gather(&directory)
        .nothing
        .expect("says so");

    assert!(
        empty_directory.contains("holds no watch list"),
        "{empty_directory}"
    );
    assert!(empty_list.contains("no path is named"), "{empty_list}");
}
