use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn_at, into, press};
use crate::ui::{Level, Screen};

#[test]
fn one_level_in_per_press_and_one_level_out_per_press() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 200, 24);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Enter);
    assert_eq!(
        app.level,
        Level::List,
        "the first press opens the panel and leaves the arrows on the list: a reader who \
         wanted to see a row has not been moved into a page of prose to get back out of"
    );
    assert!(app.detail_open);

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::Detail);
    assert!(app.detail_open);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::List);
    assert!(
        app.detail_open,
        "the detail stays beside the list, which is what makes walking the list with it open"
    );

    press(&mut app, KeyCode::Esc);
    assert_eq!(
        app.nav.at(),
        Screen::FINDINGS,
        "that press put the panel away"
    );
    assert!(!app.detail_open);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::Menu, "then the row of lists");

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn the_first_press_back_out_of_a_detail_leaves_the_panel_open_beside_the_list() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 140, 24);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, KeyCode::Left);

    assert_eq!(app.level, Level::List, "the arrows now walk the list");
    assert!(app.detail_open, "and the panel goes with them");
    assert!(
        drawn_at(&app, 140, 24).contains("THE SELECTED FINDING"),
        "{}",
        drawn_at(&app, 140, 24)
    );
}

#[test]
fn the_second_press_back_puts_the_panel_away_and_gives_the_list_the_whole_width() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 140, 24);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);

    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);

    assert_eq!(app.level, Level::List);
    assert!(!app.detail_open);
    let page = drawn_at(&app, 140, 24);
    assert!(!page.contains("THE SELECTED FINDING"), "{page}");
    assert_eq!(
        app.nav.at(),
        Screen::FINDINGS,
        "and neither press also left the section"
    );
}

#[test]
fn the_third_press_back_is_the_one_that_reaches_the_row_of_lists_and_escape_leaves_from_there() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 140, 24);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);

    for _ in 0..3 {
        press(&mut app, KeyCode::Left);
    }
    assert_eq!(app.level, Level::Menu);
    assert!(!app.detail_open);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn a_terminal_too_narrow_for_both_halves_puts_the_panel_away_on_the_first_press() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 24);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, KeyCode::Left);

    assert_eq!(app.level, Level::List);
    assert!(
        !app.detail_open,
        "there is no room to read the panel against the list, so the two rungs are one"
    );
    let page = drawn_at(&app, 80, 24);
    assert!(!page.contains("THE SELECTED FINDING"), "{page}");
    assert_eq!(app.nav.at(), Screen::FINDINGS);

    press(&mut app, KeyCode::Left);
    assert_eq!(app.level, Level::Menu, "and the next press is the rung");
}

#[test]
fn the_left_arrow_and_escape_mean_the_same_thing_on_every_rung() {
    for width in [80u16, 140] {
        let mut arrow = app();
        let mut escape = app();
        into(&mut arrow, Screen::FINDINGS, width, 24);
        into(&mut escape, Screen::FINDINGS, width, 24);
        press(&mut arrow, KeyCode::Right);
        press(&mut escape, KeyCode::Right);

        for step in 0..3 {
            if arrow.level == Level::Menu {
                break;
            }
            press(&mut arrow, KeyCode::Left);
            press(&mut escape, KeyCode::Esc);

            assert_eq!(
                drawn_at(&arrow, width, 24),
                drawn_at(&escape, width, 24),
                "at {width} columns the two keys parted company at press {}",
                step + 1
            );
        }
    }
}

#[test]
fn walking_the_list_with_the_panel_open_keeps_it_open_because_the_two_are_read_against_each_other()
{
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 140, 24);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::List);
    assert!(app.detail_open, "Escape left it beside the list");

    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Up);

    assert!(
        app.detail_open,
        "the cursor moved and the panel went with it, rather than shutting"
    );
    assert!(
        drawn_at(&app, 140, 24).contains("THE SELECTED FINDING"),
        "{}",
        drawn_at(&app, 140, 24)
    );
}
