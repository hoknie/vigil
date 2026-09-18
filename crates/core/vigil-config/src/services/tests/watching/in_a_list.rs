use crate::services::tests::harness::{changed, wrote};
use crate::{Edit, Watch, listed, put_listed, stop_listed};

const SHIPPED: &str = "\
# The files watched for a changed content, changed permissions and a new suid bit.
# Can be folder or file or file mask like /etc/ssh/*.conf
files:
  - /etc/ssh/sshd_config
  # the ones pam reads
  - /etc/pam.d
  - path: /etc/ssl/certs/ca.crt
    max_file_size: 8mb
devices:
  include: []
  exclude: []
";

const NOTHING_YET: &str = "\
# Written by this host's operator.
files: []
devices:
  include: []
  exclude: []
";

#[test]
fn the_entries_of_a_watch_list_are_read_back_with_the_size_each_one_names() {
    assert_eq!(
        listed(SHIPPED),
        vec![
            Watch::of("/etc/ssh/sshd_config", None),
            Watch::of("/etc/pam.d", None),
            Watch::of("/etc/ssl/certs/ca.crt", Some(8 * 1024 * 1024)),
        ]
    );
    assert!(listed(NOTHING_YET).is_empty());
    assert!(listed("").is_empty());
}

#[test]
fn a_path_added_to_a_watch_list_lands_after_its_last_entry_and_every_comment_stays() {
    let after = changed(put_listed(SHIPPED, &Watch::of("/etc/ssh/*.conf", None)));

    assert_eq!(
        listed(&after).last(),
        Some(&Watch::of("/etc/ssh/*.conf", None))
    );
    for kept in [
        "# Can be folder or file or file mask like /etc/ssh/*.conf",
        "  # the ones pam reads",
        "devices:",
        "  exclude: []",
    ] {
        assert!(after.contains(kept), "{kept:?} is gone from {after}");
    }
    assert!(
        after.find("\"/etc/ssh/*.conf\"") < after.find("devices:"),
        "the entry is in the list and not in the block after it: {after}"
    );
}

#[test]
fn a_path_with_its_own_size_is_written_as_the_pair_the_collector_reads_back() {
    let after = changed(put_listed(
        NOTHING_YET,
        &Watch::of("/var/lib/app/config.json", Some(512 * 1024)),
    ));

    assert!(
        after
            .contains("files:\n  - path: \"/var/lib/app/config.json\"\n    max_file_size: 512kb\n"),
        "{after}"
    );
    assert!(!after.contains("files: []"), "{after}");
    assert_eq!(
        listed(&after),
        vec![Watch::of("/var/lib/app/config.json", Some(512 * 1024))]
    );
}

#[test]
fn a_size_that_is_not_a_whole_unit_is_written_in_bytes() {
    let after = changed(put_listed(
        NOTHING_YET,
        &Watch::of("/etc/hosts", Some(1500)),
    ));

    assert!(after.contains("max_file_size: 1500\n"), "{after}");
    assert_eq!(listed(&after), vec![Watch::of("/etc/hosts", Some(1500))]);
}

#[test]
fn changing_how_an_entry_is_watched_replaces_that_entry_and_only_that_entry() {
    let resized = changed(put_listed(SHIPPED, &Watch::of("/etc/pam.d", Some(4096))));
    let plain = changed(put_listed(
        &resized,
        &Watch::of("/etc/ssl/certs/ca.crt", None),
    ));

    assert_eq!(
        listed(&plain),
        vec![
            Watch::of("/etc/ssh/sshd_config", None),
            Watch::of("/etc/pam.d", Some(4096)),
            Watch::of("/etc/ssl/certs/ca.crt", None),
        ]
    );
    assert!(
        !plain.contains("8mb"),
        "the two lines of the pair are one entry, and leaving the second behind would be a \
         size on nothing: {plain}"
    );
    assert!(plain.contains("  # the ones pam reads\n"), "{plain}");
}

#[test]
fn an_entry_already_written_as_asked_leaves_the_file_untouched() {
    assert_eq!(
        put_listed(SHIPPED, &Watch::of("/etc/pam.d", None)),
        Edit::AlreadySo
    );
    assert_eq!(
        put_listed(
            SHIPPED,
            &Watch::of("/etc/ssl/certs/ca.crt", Some(8 * 1024 * 1024))
        ),
        Edit::AlreadySo
    );
}

#[test]
fn a_list_file_with_no_list_in_it_yet_gains_one() {
    for text in ["", "devices:\n  include: []\n", "# only a comment\n"] {
        let after = changed(put_listed(text, &Watch::of("/etc/hosts", None)));
        assert_eq!(
            listed(&after),
            vec![Watch::of("/etc/hosts", None)],
            "{text:?}"
        );
    }
}

#[test]
fn a_list_written_flush_with_its_key_is_edited_at_the_column_it_was_written_at() {
    let flush = "files:\n- /etc/hosts\ndevices:\n  exclude: []\n";

    let after = changed(put_listed(flush, &Watch::of("/etc/login.defs", None)));

    assert_eq!(
        after,
        "files:\n- /etc/hosts\n- \"/etc/login.defs\"\ndevices:\n  exclude: []\n"
    );
}

#[test]
fn stopping_an_entry_takes_its_lines_away_and_nothing_else() {
    let edit = stop_listed(SHIPPED, "/etc/ssl/certs/ca.crt");
    assert_eq!(wrote(edit.clone()), 1);
    let after = changed(edit);

    assert_eq!(
        listed(&after),
        vec![
            Watch::of("/etc/ssh/sshd_config", None),
            Watch::of("/etc/pam.d", None),
        ]
    );
    assert!(!after.contains("8mb"), "{after}");
    assert!(after.contains("devices:\n  include: []\n"), "{after}");
}

#[test]
fn stopping_the_last_entry_leaves_a_list_that_watches_nothing_rather_than_a_bare_heading() {
    let one = "files:\n  - /etc/hosts\n";

    let after = changed(stop_listed(one, "/etc/hosts"));

    assert_eq!(after, "files: []\n");
}

#[test]
fn stopping_a_path_the_list_never_named_writes_nothing() {
    assert_eq!(stop_listed(SHIPPED, "/etc/nothing"), Edit::AlreadySo);
    assert_eq!(stop_listed(NOTHING_YET, "/etc/hosts"), Edit::AlreadySo);
    assert_eq!(stop_listed("", "/etc/hosts"), Edit::AlreadySo);
}

#[test]
fn a_list_written_on_one_line_is_left_to_the_person_who_wrote_it_that_way() {
    for held in ["files: [/etc/hosts]\n", "files: {a: b}\n"] {
        match put_listed(held, &Watch::of("/etc/sudoers", None)) {
            Edit::NotOurs(why) => assert!(why.contains("by hand"), "{why}"),
            other => panic!("{held}: {other:?}"),
        }
    }
}

#[test]
fn a_path_with_a_space_or_a_quotation_mark_in_it_comes_back_out_the_way_it_went_in() {
    for path in ["/etc/a file", "/etc/a\"b", "/etc/a\\b", "/etc/[ab]*.conf"] {
        let after = changed(put_listed(NOTHING_YET, &Watch::of(path, None)));
        assert_eq!(listed(&after), vec![Watch::of(path, None)], "{path}");
    }
}
