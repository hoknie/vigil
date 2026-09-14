use ratatui::crossterm::event::KeyCode;

use super::sockets::{SOCKETS, on_the_sockets};
use crate::ui::Level;
use crate::ui::app::tests::harness::{drawn_at, press};

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
    for button in ["[ x mark ]", "[ K close ]"] {
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
