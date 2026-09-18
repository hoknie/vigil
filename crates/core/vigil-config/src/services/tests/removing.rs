use super::harness::{SHIPPED, changed, entry};
use crate::{Edit, Suppression, add, remove, remove_one};

#[test]
fn a_shape_this_command_did_not_write_is_left_alone_and_named() {
    let inline = "suppressions: [{finding_key: a, reason: b}]\n";

    match add(inline, &[entry("user|group|docker")]) {
        Edit::NotOurs(said) => assert!(said.contains("by hand"), "{said}"),
        other => panic!("a file written another way must not be rewritten: {other:?}"),
    }
}

#[test]
fn what_was_written_is_taken_out_again_and_the_file_is_the_one_it_started_as() {
    let once = changed(add(SHIPPED, &[entry("user|group|docker")]));

    let after = changed(remove(&once, &["user|group|docker".to_string()]));

    assert_eq!(
        after, SHIPPED,
        "what add put in, remove takes out, down to the empty list:\n{after}"
    );
}

#[test]
fn taking_one_of_two_out_leaves_the_other_where_it_was() {
    let both = changed(add(
        SHIPPED,
        &[entry("user|group|docker"), entry("user|group|sudo")],
    ));

    let after = changed(remove(&both, &["user|group|docker".to_string()]));

    assert!(!after.contains("docker"), "{after}");
    assert!(after.contains("user|group|sudo"), "{after}");
    assert!(after.contains("suppressions:\n"), "{after}");
    assert!(!after.contains("suppressions: []"), "{after}");
}

#[test]
fn taking_out_what_is_not_there_changes_nothing_at_all() {
    assert_eq!(
        remove(SHIPPED, &["user|group|docker".to_string()]),
        Edit::AlreadySo
    );
    let once = changed(add(SHIPPED, &[entry("user|group|docker")]));
    assert_eq!(
        remove(&once, &["user|group|sudo".to_string()]),
        Edit::AlreadySo
    );
}

#[test]
fn a_comment_somebody_wrote_between_two_entries_is_not_carried_off_with_one_of_them() {
    let by_hand = "\
suppressions:
  - finding_key: \"user|group|docker\"
    reason: the deploy user belongs there
  # the staging api moves about
  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
    reason: staging
";

    let after = changed(remove(by_hand, &["user|group|docker".to_string()]));

    assert!(
        after.contains("# the staging api moves about"),
        "a line this command did not write must survive it: {after}"
    );
    assert!(after.contains("0.0.0.0:8080"), "{after}");
    assert!(!after.contains("docker"), "{after}");
}

#[test]
fn one_entry_taken_out_by_what_it_says_leaves_the_narrower_one_about_the_same_object() {
    let by_hand = "\
suppressions:
  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
    reason: staging
  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
    kind: port.listen.removed
    reason: nobody minds it going
";

    let after = changed(remove_one(
        by_hand,
        &Suppression {
            finding_key: Some("port.listen|tcp|0.0.0.0:8080".into()),
            reason: "whatever it said".into(),
            ..Suppression::default()
        },
    ));

    assert_eq!(
        after,
        "\
suppressions:
  - finding_key: \"port.listen|tcp|0.0.0.0:8080\"
    kind: port.listen.removed
    reason: nobody minds it going
"
    );
}

#[test]
fn an_entry_that_names_only_a_kind_can_be_taken_out_as_well() {
    let by_hand = "\
suppressions:
  - kind: port.listen.removed
    reason: nobody minds a service stopping here
";

    let after = changed(remove_one(
        by_hand,
        &Suppression {
            kind: Some("port.listen.removed".into()),
            reason: "any".into(),
            ..Suppression::default()
        },
    ));

    assert_eq!(after, "suppressions: []\n");
}

#[test]
fn a_prefix_is_not_taken_out_by_asking_for_the_exact_key_spelled_the_same() {
    let by_hand = "suppressions:\n  - finding_key_prefix: \"user|group|\"\n    reason: ours\n";

    assert_eq!(
        remove_one(
            by_hand,
            &Suppression {
                finding_key: Some("user|group|".into()),
                reason: "ours".into(),
                ..Suppression::default()
            }
        ),
        Edit::AlreadySo
    );
}

#[test]
fn the_first_entry_of_a_file_that_was_empty_starts_it_without_a_blank_line() {
    let after = changed(add("", &[entry("user|group|docker")]));

    assert!(
        after.starts_with("suppressions:\n  - finding_key:"),
        "{after:?}"
    );
}
