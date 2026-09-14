use super::harness::{SHIPPED, changed, entry, wrote};
use crate::{Edit, Entry, add};

#[test]
fn the_first_entry_turns_the_empty_list_into_a_block_and_moves_nothing_else() {
    let after = changed(add(SHIPPED, &[entry("port.listen|tcp|0.0.0.0:8080")]));

    assert!(after.contains("suppressions:\n"), "{after}");
    assert!(!after.contains("suppressions: []"), "{after}");
    assert!(
        after.contains("  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"\n"),
        "{after}"
    );
    assert!(
        after.contains("    reason: \"the staging api, expected here\"\n"),
        "{after}"
    );
    for kept in [
        "state_dir: /var/lib/vigil",
        "retention_days: 90",
        "  - ports",
        "# What this host is expected to do.",
        "reporters: []",
    ] {
        assert!(after.contains(kept), "{kept} was lost:\n{after}");
    }
}

#[test]
fn a_second_entry_goes_under_the_first_and_not_through_it() {
    let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

    let twice = changed(add(&once, &[entry("port.listen|tcp|0.0.0.0:8080")]));

    let docker = twice.find("user|group|docker").expect("the first is there");
    let port = twice.find("0.0.0.0:8080").expect("the second is there");
    assert!(docker < port, "{twice}");
    assert!(twice.contains("reporters: []"), "{twice}");
    assert_eq!(
        twice.matches("  - finding_key:").count(),
        2,
        "one entry per key and no more:\n{twice}"
    );
}

#[test]
fn what_is_already_written_down_is_not_written_down_twice() {
    let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

    assert_eq!(add(&once, &[entry("user|group|docker")]), Edit::AlreadySo);
}

#[test]
fn one_key_of_two_that_is_new_is_written_and_the_one_that_is_there_is_left_alone() {
    let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

    let after = changed(add(
        &once,
        &[entry("user|group|docker"), entry("user|group|sudo")],
    ));

    assert_eq!(after.matches("user|group|docker").count(), 1, "{after}");
    assert!(after.contains("user|group|sudo"), "{after}");
}

#[test]
fn one_key_named_twice_in_one_call_is_written_down_once() {
    let after = changed(add(
        SHIPPED,
        &[entry("user|group|docker"), entry("user|group|docker")],
    ));

    assert_eq!(
        after.matches("user|group|docker").count(),
        1,
        "the same object asked for twice is one entry, and a file with it twice is a file \
         an operator has to read twice to learn one thing:\n{after}"
    );
    assert_eq!(
        wrote(add(SHIPPED, &[entry("a|b"), entry("a|b"), entry("c|d")])),
        2
    );
}

#[test]
fn the_same_object_narrowed_to_two_kinds_is_two_entries_because_it_is_two_statements() {
    let one = changed(add(
        SHIPPED,
        &[Entry {
            kind: Some("port.listen.new".into()),
            ..entry("port.listen|tcp|0.0.0.0:8080")
        }],
    ));

    let both = changed(add(
        &one,
        &[Entry {
            kind: Some("port.listen.removed".into()),
            ..entry("port.listen|tcp|0.0.0.0:8080")
        }],
    ));

    assert_eq!(
        both.matches("  - finding_key:").count(),
        2,
        "silencing 'the port appeared' is not silencing 'the port is gone', and a command \
         that called the second one a duplicate would refuse to write what was asked:\n{both}"
    );
    assert_eq!(
        add(
            &both,
            &[Entry {
                kind: Some("port.listen.new".into()),
                ..entry("port.listen|tcp|0.0.0.0:8080")
            }]
        ),
        Edit::AlreadySo,
        "and the same object with the same kind is still one entry"
    );
}

#[test]
fn an_entry_written_by_hand_with_a_kind_is_read_with_it_and_not_taken_for_a_broader_one() {
    let by_hand = "\
suppressions:
  - finding_key: \"user|group|docker\"
    kind: user.group.privileged_member_added
    reason: the deploy user belongs there
";

    assert_eq!(
        add(by_hand, &[entry("user|group|docker")]),
        Edit::Changed {
            text: changed(add(by_hand, &[entry("user|group|docker")])),
            entries: 1,
        },
        "an entry that silences one kind does not silence the object, so the broader one \
         is a new entry and not a duplicate"
    );
}

#[test]
fn a_prefix_and_an_exact_key_are_two_different_entries_and_neither_hides_the_other() {
    let exact = changed(add(SHIPPED, &[entry("port.listen|tcp|10.0.0.5:")]));
    let both = changed(add(
        &exact,
        &[Entry {
            prefix: true,
            ..entry("port.listen|tcp|10.0.0.5:")
        }],
    ));

    assert!(both.contains("  - finding_key: \"port.listen|tcp|10.0.0.5:\""));
    assert!(both.contains("  - finding_key_prefix: \"port.listen|tcp|10.0.0.5:\""));
}

#[test]
fn a_kind_and_a_date_are_written_beside_the_key_that_carries_them() {
    let after = changed(add(
        SHIPPED,
        &[Entry {
            kind: Some("port.listen.new".into()),
            until: Some("2026-12-31T00:00:00.000Z".into()),
            ..entry("port.listen|tcp|0.0.0.0:8080")
        }],
    ));

    assert!(after.contains("    kind: \"port.listen.new\"\n"), "{after}");
    assert!(
        after.contains("    until: \"2026-12-31T00:00:00.000Z\"\n"),
        "{after}"
    );
}

#[test]
fn a_file_that_never_had_the_key_gets_a_block_of_its_own_at_the_end() {
    let bare = "state_dir: /var/lib/vigil\n";

    let after = changed(add(bare, &[entry("user|group|docker")]));

    assert!(after.starts_with("state_dir: /var/lib/vigil\n"), "{after}");
    assert!(
        after.contains("\nsuppressions:\n  - finding_key:"),
        "{after}"
    );
}
