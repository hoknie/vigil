use super::harness::{busy, drawn, drawn_with, showing};
use crate::ui::Protocols;
use crate::ui::screens::ports::Arrangement;

#[test]
fn a_narrowed_screen_counts_what_is_on_it_as_well_as_what_the_agent_read() {
    let whole = drawn(&busy(), 0, 100);
    assert!(whole.contains("4 listening socket(s)"), "{whole}");
    assert!(!whole.contains(" of 4 "), "{whole}");

    let mut protocols = Protocols::default();
    protocols.toggle('u');
    let narrowed = drawn_with(&busy(), &showing(protocols, Arrangement::Flat), 0, 100);
    assert!(
        narrowed.contains("3 of 4 listening socket(s)"),
        "{narrowed}"
    );
}

#[test]
fn a_socket_whose_owner_could_not_be_looked_up_says_so_instead_of_showing_nothing() {
    let page = drawn(&busy(), 0, 80);

    assert!(page.contains("owner not resolved"), "{page}");
    assert!(page.contains("1 whose process is not visible"), "{page}");
}
