use crate::services::tests::harness::{changed, wrote};
use crate::{Edit, Watch, paths, put, stop};

const WITH_A_LIST: &str = "\
state_dir: /var/lib/vigil

files:
  paths:
    - \"/etc/hosts\"
    - path: \"/etc/ssl/certs/ca-certificates.crt\"
      ceiling_bytes: 8388608
  ceiling_bytes: 1048576

reporters: []
";

const WITH_NOTHING_WATCHED: &str = "\
state_dir: /var/lib/vigil

files:
  paths: []

reporters: []
";

const WITH_A_BLOCK_AND_NO_LIST: &str = "\
files:
  ceiling_bytes: 1048576
";

#[test]
fn the_paths_a_file_names_are_read_back_with_the_ceiling_each_one_was_given() {
    let held = paths(WITH_A_LIST);

    assert_eq!(
        held,
        vec![
            Watch::of("/etc/hosts", None),
            Watch::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608)),
        ]
    );
}

#[test]
fn a_file_that_names_no_files_block_at_all_names_no_watched_path() {
    assert!(paths("state_dir: /var/lib/vigil\n").is_empty());
    assert!(paths(WITH_NOTHING_WATCHED).is_empty());
    assert!(paths("").is_empty());
}

#[test]
fn a_path_added_to_a_list_lands_at_the_end_of_it_and_leaves_every_other_line_alone() {
    let after = changed(put(WITH_A_LIST, &Watch::of("/etc/sudoers", None)));

    assert_eq!(
        paths(&after),
        vec![
            Watch::of("/etc/hosts", None),
            Watch::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608)),
            Watch::of("/etc/sudoers", None),
        ]
    );
    assert!(after.contains("ceiling_bytes: 1048576"), "{after}");
    assert!(
        after.contains("state_dir: /var/lib/vigil"),
        "every other key of a file an operator keeps in version control stays where it was: \
         {after}"
    );
}

#[test]
fn a_path_added_to_a_file_that_watches_nothing_turns_the_empty_list_into_a_block() {
    let after = changed(put(
        WITH_NOTHING_WATCHED,
        &Watch::of("/etc/sudoers", Some(4096)),
    ));

    assert_eq!(paths(&after), vec![Watch::of("/etc/sudoers", Some(4096))]);
    assert!(!after.contains("paths: []"), "{after}");
}

#[test]
fn a_path_added_to_a_file_with_no_files_block_writes_the_block_the_daemon_reads() {
    let after = changed(put(
        "state_dir: /var/lib/vigil\n",
        &Watch::of("/etc/sudoers", None),
    ));

    assert_eq!(paths(&after), vec![Watch::of("/etc/sudoers", None)]);
    assert!(after.contains("files:"), "{after}");
}

#[test]
fn a_path_added_to_a_block_that_names_no_list_keeps_what_the_block_already_said() {
    let after = changed(put(
        WITH_A_BLOCK_AND_NO_LIST,
        &Watch::of("/etc/sudoers", None),
    ));

    assert_eq!(paths(&after), vec![Watch::of("/etc/sudoers", None)]);
    assert!(
        after.contains("ceiling_bytes: 1048576"),
        "the ceiling the block names is what every path without one of its own is hashed to, \
         and losing it would quietly change how the whole list is watched: {after}"
    );
}

#[test]
fn a_path_already_written_exactly_as_it_is_asked_for_leaves_the_file_untouched() {
    assert_eq!(
        put(WITH_A_LIST, &Watch::of("/etc/hosts", None)),
        Edit::AlreadySo
    );
    assert_eq!(
        put(
            WITH_A_LIST,
            &Watch::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608))
        ),
        Edit::AlreadySo
    );
}

#[test]
fn changing_how_a_path_is_watched_replaces_that_entry_and_only_that_entry() {
    let after = changed(put(WITH_A_LIST, &Watch::of("/etc/hosts", Some(2048))));

    assert_eq!(
        paths(&after),
        vec![
            Watch::of("/etc/hosts", Some(2048)),
            Watch::of("/etc/ssl/certs/ca-certificates.crt", Some(8_388_608)),
        ]
    );
}

#[test]
fn taking_a_ceiling_off_a_path_leaves_the_path_written_as_the_word_it_started_as() {
    let after = changed(put(
        WITH_A_LIST,
        &Watch::of("/etc/ssl/certs/ca-certificates.crt", None),
    ));

    assert_eq!(
        paths(&after),
        vec![
            Watch::of("/etc/hosts", None),
            Watch::of("/etc/ssl/certs/ca-certificates.crt", None),
        ]
    );
    assert!(
        !after.contains("8388608"),
        "the two lines of the pair are one entry, and leaving the second behind would be a \
         ceiling on nothing: {after}"
    );
}

#[test]
fn a_path_the_agent_watches_that_the_file_never_named_is_written_down_rather_than_refused() {
    let after = changed(put(WITH_A_LIST, &Watch::of("/etc/login.defs", Some(2048))));

    assert_eq!(
        paths(&after).last(),
        Some(&Watch::of("/etc/login.defs", Some(2048))),
        "the list a reader edits is what the agent watches, and the paths this product ships \
         are watched without the file naming one of them: changing how such a path is watched \
         is the file naming it for the first time"
    );
}

#[test]
fn a_path_stopped_leaves_the_rest_of_the_list_and_the_rest_of_the_file_as_they_were() {
    let edit = stop(WITH_A_LIST, "/etc/ssl/certs/ca-certificates.crt");
    assert_eq!(wrote(edit.clone()), 1);
    let after = changed(edit);

    assert_eq!(paths(&after), vec![Watch::of("/etc/hosts", None)]);
    assert!(after.contains("reporters: []"), "{after}");
}

#[test]
fn stopping_the_last_path_leaves_a_list_that_watches_nothing_rather_than_a_heading_with_no_items() {
    let one = changed(stop(WITH_A_LIST, "/etc/ssl/certs/ca-certificates.crt"));
    let none = changed(stop(&one, "/etc/hosts"));

    assert!(none.contains("paths: []"), "{none}");
    assert!(paths(&none).is_empty());
}

#[test]
fn stopping_a_path_this_file_never_named_writes_nothing_at_all() {
    assert_eq!(stop(WITH_A_LIST, "/etc/nothing"), Edit::AlreadySo);
    assert_eq!(stop("state_dir: /var\n", "/etc/hosts"), Edit::AlreadySo);
    assert_eq!(stop(WITH_NOTHING_WATCHED, "/etc/hosts"), Edit::AlreadySo);
}

#[test]
fn a_list_written_on_one_line_is_left_to_the_person_who_wrote_it_that_way() {
    for held in [
        "files:\n  paths: [\"/etc/hosts\"]\n",
        "files: { paths: [] }\n",
    ] {
        match put(held, &Watch::of("/etc/sudoers", None)) {
            Edit::NotOurs(why) => assert!(why.contains("--force"), "{why}"),
            other => panic!("{held}: {other:?}"),
        }
    }
}

#[test]
fn a_path_with_a_space_or_a_quotation_mark_in_it_comes_back_out_the_way_it_went_in() {
    for path in ["/etc/a file", "/etc/a\"b", "/etc/a\\b", "/etc/hosts"] {
        let after = changed(put(WITH_NOTHING_WATCHED, &Watch::of(path, None)));
        assert_eq!(paths(&after), vec![Watch::of(path, None)], "{path}");
    }
}

#[test]
fn a_comment_an_operator_wrote_in_the_list_is_not_taken_for_a_path() {
    let commented = "files:\n  paths:\n    # the ones we care about\n    - \"/etc/hosts\"\n";

    assert_eq!(paths(commented), vec![Watch::of("/etc/hosts", None)]);
}
