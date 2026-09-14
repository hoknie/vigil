use ratatui::crossterm::event::KeyCode;

use super::harness::{app, drawn_at, number, press};
use crate::ui::Level;
use crate::ui::fixture::screen;

const SOCKETS: usize = 0;

const BY_PROGRAM: usize = 1;

fn on_the_sockets(pane: usize) -> crate::ui::App {
    let mut app = app();
    press(&mut app, number(screen("ports")));
    drawn_at(&app, 120, 30);
    for _ in 0..pane {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    app
}

#[test]
fn a_tree_arrives_folded_and_one_arrow_opens_one_branch_and_the_next_opens_the_row() {
    let mut app = on_the_sockets(BY_PROGRAM);

    let folded = drawn_at(&app, 120, 30);
    assert!(folded.contains("nginx"), "{folded}");
    assert!(!folded.contains(":::443"), "{folded}");

    press(&mut app, KeyCode::Right);
    let opened = drawn_at(&app, 120, 30);
    assert!(
        opened.contains(":::443") || opened.contains("0.0.0.0"),
        "the first arrow shows what is under the heading: {opened}"
    );
    assert_eq!(
        app.level,
        Level::List,
        "and it is still the list that has the arrows, not the detail panel"
    );

    press(&mut app, KeyCode::Right);
    assert!(
        drawn_at(&app, 120, 30).contains("THE SELECTED ROW"),
        "the second arrow shows the panel"
    );
    assert_eq!(
        app.level,
        Level::List,
        "and the arrows are still the list's: a reader who wanted the next row has not been \
         moved into a page of prose to get back out of"
    );

    press(&mut app, KeyCode::Right);
    assert_eq!(
        app.level,
        Level::Detail,
        "the third arrow is the one that hands the arrows to the panel"
    );
}

#[test]
fn the_left_arrow_undoes_that_in_the_same_order_it_was_done() {
    let mut app = on_the_sockets(BY_PROGRAM);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, KeyCode::Left);
    assert_eq!(
        app.level,
        Level::List,
        "the arrows come back to the list first, with the panel still up"
    );

    press(&mut app, KeyCode::Left);
    assert!(
        !drawn_at(&app, 120, 30).contains("THE SELECTED ROW"),
        "then the panel goes away, with the branch still open"
    );
    assert!(drawn_at(&app, 120, 30).contains('\u{25be}'), "still open");

    press(&mut app, KeyCode::Left);
    let folded = drawn_at(&app, 120, 30);
    assert!(
        !folded.contains('\u{25be}'),
        "and only then does the branch fold away: {folded}"
    );
}

#[test]
fn a_socket_under_an_open_branch_folds_it_and_lands_the_cursor_on_the_heading() {
    let mut app = on_the_sockets(BY_PROGRAM);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Left);

    let page = drawn_at(&app, 120, 30);
    assert!(
        !page.contains('\u{25be}'),
        "the branch the cursor was inside is the one that folds: {page}"
    );
    assert!(
        page.lines()
            .any(|line| line.contains(" > ") && line.contains('\u{25b8}')),
        "and the cursor is left on the heading it came out of, not on a row that moved: {page}"
    );
}

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

#[test]
fn the_panel_of_a_socket_draws_its_actions_where_every_panel_draws_them() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Right);

    let page = drawn_at(&app, 200, 30);
    let actions = page
        .lines()
        .position(|line| line.contains("ACTIONS"))
        .expect("an actions section");
    let suppress = page
        .lines()
        .position(|line| line.contains("SUPPRESS"))
        .expect("a suppress section");

    assert!(
        actions < suppress,
        "the panel is one template on every screen, and the actions sit above the block an \
         operator copies out of it: {page}"
    );
    for button in ["[ K close this socket ]", "[ S suppress it ]"] {
        assert!(page.contains(button), "{button} is missing: {page}");
    }
}

#[test]
fn every_arrow_into_the_panel_has_an_arrow_back_out_of_it_in_the_same_place() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);
    assert_eq!(
        app.button_at(),
        None,
        "the panel has the arrows and they scroll it; the buttons are a step further in"
    );

    press(&mut app, KeyCode::Right);
    assert_eq!(app.button_at(), Some(0), "onto the first button");
    press(&mut app, KeyCode::Right);
    assert_eq!(app.button_at(), Some(1), "along the row");
    press(&mut app, KeyCode::Right);
    assert_eq!(
        app.button_at(),
        Some(1),
        "and it stops at the end rather than wrapping, so every step in has a step out"
    );

    press(&mut app, KeyCode::Left);
    assert_eq!(app.button_at(), Some(0));
    press(&mut app, KeyCode::Left);
    assert_eq!(
        app.button_at(),
        None,
        "back to the panel itself, which is where the arrows came in from"
    );
    assert_eq!(app.level, Level::Detail, "and not out of it in one press");

    press(&mut app, KeyCode::Left);
    assert_eq!(app.level, Level::List, "the last one leaves");
}

#[test]
fn the_arrows_scroll_the_panel_wherever_they_are_inside_it() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.button_at(), Some(0));
    drawn_at(&app, 200, 24);

    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);

    let page = drawn_at(&app, 200, 24);
    assert!(
        page.contains("lines 3-"),
        "up and down scroll the panel whether the arrows are on its body or on its \
         buttons: {page}"
    );
    assert_eq!(
        app.button_at(),
        Some(0),
        "and they leave the buttons where they were"
    );
}

#[test]
fn enter_on_a_button_does_what_the_button_says() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.button_at(), Some(0));

    press(&mut app, KeyCode::Enter);

    assert!(
        app.choosing(),
        "the first button closes the socket, and pressing it opens the band that asks how"
    );
}

#[test]
fn a_socket_can_be_closed_from_its_panel_without_marking_it_first() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Right);
    assert!(app.marked().is_empty());

    press(&mut app, KeyCode::Char('K'));

    assert!(
        app.choosing(),
        "the panel is open on one socket, and asking to close it must not send the reader \
         back to the list to mark it first"
    );
    assert_eq!(app.what_a_kill_would_take().len(), 1);
}

#[test]
fn the_findings_screen_is_left_as_it_was_because_nothing_on_it_can_be_closed() {
    let mut app = super::harness::app();
    press(
        &mut app,
        super::harness::number(crate::ui::Screen::FINDINGS),
    );
    drawn_at(&app, 120, 40);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Right);

    let page = drawn_at(&app, 120, 40);

    assert!(page.contains("SUPPRESS"), "{page}");
    for button in ["ACTIONS", "[ x mark ]", "[ K close ]"] {
        assert!(
            !page.contains(button),
            "a finding is not a socket: {button} is on a screen it does not belong to: {page}"
        );
    }
}

#[test]
fn every_letter_the_band_draws_reaches_it_even_where_that_letter_means_something_else() {
    for key in ['S', 'K', 'D'] {
        let mut app = on_the_sockets(SOCKETS);
        press(&mut app, KeyCode::Char('x'));
        press(&mut app, KeyCode::Char('K'));
        assert!(app.choosing(), "{key}: the band did not open");

        press(&mut app, KeyCode::Char(key));

        assert!(
            !app.choosing(),
            "{key} is drawn in the band and did nothing. K is the key that opens the band \
             everywhere else on this screen, and a band that reads its letters through the \
             screen's own key map loses exactly the one that matters most"
        );
    }
}

#[test]
fn the_key_that_walks_away_from_the_band_leaves_the_host_and_the_marks_alone() {
    let mut app = on_the_sockets(SOCKETS);
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Char('K'));

    press(&mut app, KeyCode::Char('C'));

    assert!(!app.choosing());
    assert_eq!(app.marked().len(), 1);
}
