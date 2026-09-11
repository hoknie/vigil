use super::harness::{degrade, drawn_at};
use crate::ui::{Subject, fixture};

#[test]
fn the_tally_counts_the_list_and_not_the_whole_reading() {
    let page = drawn_at(&fixture::view(), Subject::Sudo, 120);
    let last = page.lines().last().expect("a tally").to_string();

    assert!(last.contains("1 sudo grant,"), "{last}");
}

#[test]
fn a_collector_that_could_not_read_everything_says_so_under_the_table() {
    let mut view = fixture::view();
    degrade(&mut view, "/etc/shadow is not readable; run as root");

    let page = drawn_at(&view, Subject::Users, 200);

    assert!(page.contains("incomplete"), "{page}");
    assert!(page.contains("/etc/shadow is not readable"), "{page}");
}
