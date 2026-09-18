use std::path::Path;

use crate::{Switched, blocks_in, new_block, switched, with_block};

const SHIPPED: &str = "\
users:
  # How often collector reads, in seconds.
  schedule: 300

  accounts:
    from_the_console: false
";

const TWO: &str = "\
containers:
  schedule: 60
---
containers-engines:
  # How often collector reads, in seconds.
  schedule: 120
  engines: [docker, podman]
";

fn changed(switched: Switched) -> String {
    match switched {
        Switched::Changed(text) => text,
        other => panic!("{other:?}"),
    }
}

fn enabled_of(text: &str, name: &str) -> bool {
    blocks_in(Path::new("x.yaml"), text)
        .expect("what was written still reads")
        .into_iter()
        .find(|block| block.name == name)
        .expect("the block is still there")
        .enabled
}

#[test]
fn switching_a_block_off_adds_one_line_under_its_key_and_touches_no_other() {
    let after = changed(switched(SHIPPED, "users", false));

    assert_eq!(
        after,
        SHIPPED.replacen("users:\n", "users:\n  enabled: false\n", 1),
        "the comments, the blank line and the verb stay where the operator put them"
    );
    assert!(!enabled_of(&after, "users"));
}

#[test]
fn switching_it_back_on_edits_the_line_in_place_and_keeps_what_is_beside_it() {
    let off = "users:\n  enabled: false   # until the audit is over\n  schedule: 300\n";

    let after = changed(switched(off, "users", true));

    assert_eq!(
        after,
        "users:\n  enabled: true   # until the audit is over\n  schedule: 300\n"
    );
    assert!(enabled_of(&after, "users"));
}

#[test]
fn a_block_that_already_says_what_is_asked_is_not_written_again() {
    assert_eq!(switched(SHIPPED, "users", true), Switched::AlreadySo);
    assert_eq!(
        switched("users:\n  enabled: false\n", "users", false),
        Switched::AlreadySo
    );
    assert_eq!(
        switched("users:\n  enabled: true\n", "users", true),
        Switched::AlreadySo
    );
}

#[test]
fn the_block_of_a_second_document_is_found_and_the_first_is_left_alone() {
    let after = changed(switched(TWO, "containers-engines", false));

    assert!(enabled_of(&after, "containers"));
    assert!(!enabled_of(&after, "containers-engines"));
    assert_eq!(after.lines().count(), TWO.lines().count() + 1, "{after}");
    assert!(
        after.contains("containers-engines:\n  enabled: false\n  # How often"),
        "{after}"
    );
}

#[test]
fn a_key_that_only_begins_like_the_name_is_not_taken_for_it() {
    let after = changed(switched(TWO, "containers", false));

    assert!(!enabled_of(&after, "containers"));
    assert!(enabled_of(&after, "containers-engines"));
}

#[test]
fn a_block_with_nothing_in_it_is_given_the_line_at_the_usual_indent() {
    let after = changed(switched("network:\nusers:\n", "network", false));

    assert_eq!(after, "network:\n  enabled: false\nusers:\n");
    assert!(enabled_of(&after, "users"));
}

#[test]
fn the_indent_the_operator_chose_is_the_indent_the_new_line_gets() {
    let after = changed(switched("files:\n    schedule: 300\n", "files", false));

    assert_eq!(after, "files:\n    enabled: false\n    schedule: 300\n");
}

#[test]
fn an_enabled_deeper_in_the_block_is_not_the_blocks_own() {
    let text = "files:\n  report:\n    enabled: true\n";

    let after = changed(switched(text, "files", false));

    assert_eq!(
        after,
        "files:\n  enabled: false\n  report:\n    enabled: true\n"
    );
}

#[test]
fn a_shape_this_command_did_not_write_is_left_alone_and_named() {
    for (text, name) in [
        ("users: {schedule: 300}\n", "users"),
        ("network:\n  schedule: 30\n", "users"),
        ("users:\n  enabled: maybe\n", "users"),
    ] {
        match switched(text, name, false) {
            Switched::NotOurs(said) => assert!(said.contains("by hand"), "{said}"),
            other => panic!("{text:?}: {other:?}"),
        }
    }
}

#[test]
fn a_file_with_no_newline_at_its_end_is_given_none() {
    let after = changed(switched("users:\n  schedule: 300", "users", false));

    assert_eq!(after, "users:\n  enabled: false\n  schedule: 300");
}

#[test]
fn a_new_block_names_its_period_and_reads_back_as_a_collector_switched_on() {
    let written = new_block("firewall", 60);

    let read = blocks_in(Path::new("firewall.yaml"), &written).expect("reads");
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].name, "firewall");
    assert!(read[0].enabled);
    assert_eq!(read[0].schedule, Some(60));
}

#[test]
fn a_block_added_to_a_file_that_holds_others_leaves_them_as_they_were() {
    let after = with_block(SHIPPED, &new_block("processes", 30));

    assert!(after.starts_with(SHIPPED.trim_end()), "{after}");
    let names: Vec<String> = blocks_in(Path::new("x.yaml"), &after)
        .expect("reads")
        .into_iter()
        .map(|block| block.name)
        .collect();
    assert_eq!(names, ["users", "processes"]);
    assert_eq!(with_block("", "users:\n"), "users:\n");
}
