use super::harness::{SHIPPED, changed, entry};
use crate::{Edit, add, named, remove};

#[test]
fn an_entry_written_by_hand_without_quotes_is_still_found_by_its_key() {
    let by_hand = "suppressions:\n  - finding_key: user|group|docker\n    reason: ours\n";

    assert_eq!(named(by_hand), vec!["user|group|docker".to_string()]);
    assert_eq!(add(by_hand, &[entry("user|group|docker")]), Edit::AlreadySo);
    assert!(matches!(
        remove(by_hand, &["user|group|docker".to_string()]),
        Edit::Changed { .. }
    ));
}

#[test]
fn an_entry_whose_key_is_not_on_the_first_line_of_it_is_read_all_the_same() {
    let by_hand = "suppressions:\n  - reason: ours\n    finding_key: \"user|group|docker\"\n";

    assert_eq!(named(by_hand), vec!["user|group|docker".to_string()]);
}

#[test]
fn a_quotation_mark_in_a_key_does_not_end_the_string_early_and_comes_back_whole() {
    let odd = "user|sshkey|deploy|a\"b";

    let after = changed(add(SHIPPED, &[entry(odd)]));

    assert!(after.contains("\"user|sshkey|deploy|a\\\"b\""), "{after}");
    assert_eq!(named(&after), vec![odd.to_string()]);
}
