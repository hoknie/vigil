use ratatui::crossterm::event::KeyCode;

use super::sockets::{BY_PROGRAM, SOCKETS, on_the_sockets};
use crate::ui::app::tests::harness::{drawn_at, press};

#[test]
fn marking_a_socket_draws_a_character_against_it_and_marking_it_again_takes_it_off() {
    let mut app = on_the_sockets(SOCKETS);

    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.marked().len(), 1);

    press(&mut app, KeyCode::Char('x'));
    assert!(app.marked().is_empty());
}

#[test]
fn what_is_marked_in_one_list_of_the_section_is_not_marked_in_the_other() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.marked().len(), 1);

    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);

    assert!(
        app.marked().is_empty(),
        "a kill is aimed at rows a person is looking at"
    );
}

#[test]
fn with_nothing_marked_the_key_takes_the_row_the_cursor_is_on() {
    let mut app = on_the_sockets(SOCKETS);
    assert!(app.marked().is_empty());

    press(&mut app, KeyCode::Char('K'));

    assert!(
        app.choosing(),
        "the cursor is on a socket, so there is one to close"
    );
    assert_eq!(app.what_a_kill_would_take().len(), 1);
}

#[test]
fn on_a_heading_with_nothing_marked_there_is_nothing_to_close_and_it_says_so() {
    let mut app = on_the_sockets(BY_PROGRAM);
    assert!(app.marked().is_empty());

    press(&mut app, KeyCode::Char('K'));

    assert!(
        !app.choosing(),
        "a heading is a line this console drew, not a socket"
    );
    assert!(
        drawn_at(&app, 120, 30).contains("nothing here to close"),
        "{}",
        drawn_at(&app, 120, 30)
    );
}

#[test]
fn the_band_that_kills_offers_the_three_ways_and_says_what_enter_costs() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));

    press(&mut app, KeyCode::Char('K'));
    let ways = drawn_at(&app, 120, 30);
    assert!(ways.contains("SIGTERM"), "{ways}");
    assert!(ways.contains("SIGKILL"), "{ways}");
    assert!(ways.contains("leave the process running"), "{ways}");
    assert!(
        ways.contains("on this host, now"),
        "there is no second band after this one, so this is where Enter has to be spelled \
         out: {ways}"
    );
}

#[test]
fn escape_leaves_the_band_without_touching_the_host_and_keeps_what_was_marked() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Char('K'));

    press(&mut app, KeyCode::Esc);

    assert!(!app.choosing());
    assert_eq!(app.marked().len(), 1, "what was marked is still marked");
}

#[test]
fn the_suppressions_for_what_is_marked_are_one_block_keyed_by_the_finding_key() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('x'));

    press(&mut app, KeyCode::Char('S'));

    let page = drawn_at(&app, 120, 30);
    assert!(page.contains("suppressions:"), "{page}");
    assert_eq!(
        page.matches("finding_key:").count(),
        2,
        "both marked rows are in the block: {page}"
    );
    assert!(
        page.contains("port.listen|"),
        "the key is the one the agent writes findings against, not the row key: {page}"
    );
}

#[test]
fn a_sheet_the_console_put_up_is_taken_down_by_the_next_key_and_eats_it() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Char('S'));
    assert!(drawn_at(&app, 120, 30).contains("suppressions:"));

    press(&mut app, KeyCode::Char('x'));

    assert!(!drawn_at(&app, 120, 30).contains("suppressions:"));
    assert_eq!(
        app.marked().len(),
        1,
        "the key that closed the sheet must not also do what it usually does"
    );
}

#[test]
fn marking_a_program_takes_every_socket_under_it_and_never_the_heading_itself() {
    let mut app = on_the_sockets(BY_PROGRAM);

    press(&mut app, KeyCode::Char('x'));

    let marked = app.marked();
    assert!(!marked.is_empty(), "a heading of sockets marked nothing");
    assert!(
        marked.iter().all(|key| !key.starts_with("program|")),
        "the heading is a line this console drew, not a socket the agent read, and a kill \
         aimed at it aims at nothing: {marked:?}"
    );
}

#[test]
fn a_program_marked_folded_marks_the_same_sockets_as_one_marked_open() {
    let mut folded = on_the_sockets(BY_PROGRAM);
    press(&mut folded, KeyCode::Char('x'));

    let mut opened = on_the_sockets(BY_PROGRAM);
    press(&mut opened, KeyCode::Right);
    press(&mut opened, KeyCode::Char('x'));

    assert_eq!(
        folded.marked(),
        opened.marked(),
        "a reader who marked a folded program and one who opened it first have said the same \
         thing, and a console where they have not is a console where folding changes what a \
         kill hits"
    );
}

#[test]
fn unmarking_everything_leaves_nothing_for_the_next_kill_to_find() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.marked().len(), 2);

    press(&mut app, KeyCode::Char('M'));

    assert!(app.marked().is_empty());
}

#[test]
fn the_key_row_of_a_list_a_reader_can_act_on_names_those_keys_at_the_foot_of_the_screen() {
    let app = on_the_sockets(SOCKETS);

    for width in [80u16, 120, 200] {
        let page = drawn_at(&app, width, 24);
        let foot = page.lines().last().expect("a key row").to_string();

        assert!(foot.contains("x mark"), "{width}: {foot}");
        assert!(foot.contains("K close"), "{width}: {foot}");
        assert!(
            foot.contains("f filter"),
            "the same key narrows the findings, and a reader who learned it there must find \
             it here: {width}: {foot}"
        );
    }
}
