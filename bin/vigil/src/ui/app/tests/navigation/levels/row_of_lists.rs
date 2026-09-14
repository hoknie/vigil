use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn, into, number, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

#[test]
fn the_left_arrow_on_the_row_of_lists_walks_along_it_and_never_out_of_the_section() {
    for screen in [
        screen("ports"),
        screen("accounts"),
        screen("programs"),
        screen("startup"),
    ] {
        let mut app = app();
        into(&mut app, screen, 140, 24);
        press(&mut app, KeyCode::Right);
        press(&mut app, KeyCode::Right);

        press(&mut app, KeyCode::Left);
        assert_eq!(app.level, Level::List, "{}", screen.name());
        press(&mut app, KeyCode::Left);
        assert_eq!(app.level, Level::List, "{}", screen.name());
        press(&mut app, KeyCode::Left);
        assert_eq!(app.level, Level::Menu, "{}", screen.name());

        for _ in 0..3 {
            press(&mut app, KeyCode::Left);
        }

        assert_eq!(
            app.nav.at(),
            screen,
            "the left arrow walked out of {}",
            screen.name()
        );
    }
}

#[test]
fn the_arrows_move_whatever_has_them_and_nothing_else() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 200, 24);

    press(&mut app, KeyCode::Down);
    assert_eq!(app.nav.findings.at(), 1);
    assert_eq!(app.nav.difference.top(), 0);

    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.nav.findings.at(), 1, "the cursor stayed where it was");
    assert!(app.nav.difference.top() > 0, "and the detail moved instead");
}

#[test]
fn the_summary_has_nothing_behind_its_lines_so_right_is_the_end_of_travel() {
    let mut app = app();
    into(&mut app, Screen::SUMMARY, 200, 24);

    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.level, Level::List);
    assert!(!app.detail_open);
}

#[test]
fn a_section_with_a_row_of_lists_puts_it_above_the_rows_and_below_nothing() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));
    assert_eq!(app.level, Level::Menu, "it is the top rung of the section");
    assert!(
        drawn(&app).contains("[users]"),
        "and the row of them is on the screen: {}",
        drawn(&app)
    );

    press(&mut app, KeyCode::Down);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::Menu, "and back out one rung at a time");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn the_horizontal_arrows_walk_the_row_of_lists_and_leave_the_section_alone() {
    let mut app = app();
    press(&mut app, number(screen("accounts")));

    press(&mut app, KeyCode::Right);
    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert_eq!(
        app.nav.at(),
        screen("accounts"),
        "the section did not change"
    );
    assert!(drawn(&app).contains("[groups]"), "{}", drawn(&app));

    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.panes().expect("a section").showing(), 5, "and it wraps");
}
