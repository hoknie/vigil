use super::harness::{busy, drawn, drawn_with, looking_for, showing};
use crate::ui::Protocols;
use crate::ui::screens::ports::Arrangement;

#[test]
fn the_protocol_chooser_is_on_the_screen_before_anybody_touches_it() {
    let page = drawn(&busy(), 0, 80);

    assert!(page.contains("[t tcp]"), "{page}");
    assert!(page.contains("[x unix]"), "{page}");
    assert!(page.contains("a all"), "{page}");
}

#[test]
fn hiding_a_protocol_takes_its_rows_off_the_screen_and_says_so_under_the_table() {
    let mut protocols = Protocols::default();
    protocols.toggle('u');

    let page = drawn_with(
        &busy(),
        &showing(protocols.clone(), Arrangement::Flat),
        0,
        80,
    );

    assert!(page.contains("0.0.0.0:80"), "{page}");
    assert!(!page.contains("0.0.0.0:53"), "the udp row went: {page}");
    assert!(
        page.contains(" u udp ") && !page.contains("[u udp]"),
        "the chooser says which kinds are off: {page}"
    );
    assert!(
        drawn_with(&busy(), &showing(protocols, Arrangement::Flat), 0, 120).contains("hiding udp"),
        "and so does the line under the table, where there is room for it"
    );
}

#[test]
fn a_table_emptied_by_the_filter_is_not_a_host_with_nothing_listening() {
    let mut protocols = Protocols::default();
    for (key, _) in crate::ui::types::filters::protocols::ALL {
        protocols.toggle(*key);
    }

    let page = drawn_with(
        &busy(),
        &showing(protocols.clone(), Arrangement::Flat),
        0,
        80,
    );

    assert!(page.contains("not by the agent"), "{page}");
    assert!(!page.contains("found no listening sockets"), "{page}");
}

#[test]
fn a_search_narrows_the_table_and_an_empty_result_names_what_emptied_it() {
    let page = drawn_with(&busy(), &looking_for("dnsmasq"), 0, 80);
    assert!(page.contains("0.0.0.0:53"), "{page}");
    assert!(!page.contains("0.0.0.0:80"), "{page}");
    assert!(page.contains("search"), "{page}");

    let nothing = drawn_with(&busy(), &looking_for("nothing like this"), 0, 80);
    assert!(
        nothing.contains("Nothing the agent read here matches"),
        "{nothing}"
    );
    assert!(
        nothing.contains("every value recorded about a socket"),
        "and what it looked at: {nothing}"
    );

    let mut both = looking_for("nothing like this");
    both.protocols.toggle('u');
    let both = drawn_with(&busy(), &both, 0, 80);
    assert!(both.contains("among the kinds you are showing"), "{both}");
    assert!(both.contains("Two filters are on"), "{both}");
}

#[test]
fn the_search_reaches_the_path_and_the_command_line_the_table_has_no_room_for() {
    for wanted in ["/usr/sbin/dnsmasq", "dnsmasq -k", "0.0.0.0:53", "udp"] {
        let page = drawn_with(&busy(), &looking_for(wanted), 0, 80);
        assert!(
            page.contains("0.0.0.0:53"),
            "{wanted} found nothing: {page}"
        );
    }
}
