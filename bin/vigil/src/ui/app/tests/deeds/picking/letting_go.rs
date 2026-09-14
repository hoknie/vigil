use ratatui::crossterm::event::KeyCode;

use super::findings::{on_the_findings, shift};
use crate::ui::app::tests::harness::{app, drawn, drawn_at, into, press};
use crate::ui::{Level, Screen};

#[test]
fn escape_lets_go_of_what_is_picked_before_it_leaves_the_screen() {
    let mut app = on_the_findings();
    shift(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::FINDINGS, "it left the screen as well");
    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn walking_off_the_findings_lets_go_of_the_rows_that_cannot_be_seen_any_more() {
    let mut app = on_the_findings();
    press(&mut app, KeyCode::Char('a'));

    press(&mut app, KeyCode::Char('1'));
    press(&mut app, KeyCode::Char('9'));

    assert!(
        !drawn(&app).contains("picked"),
        "a deed must not reach rows a reader picked on a screen they have left: {}",
        drawn(&app)
    );
}

#[test]
fn a_search_that_hides_a_picked_row_lets_go_of_it_rather_than_acting_on_it_unseen() {
    let mut app = on_the_findings();
    press(&mut app, KeyCode::Char('a'));

    press(&mut app, KeyCode::Char('/'));
    for character in "logged in".chars() {
        press(&mut app, KeyCode::Char(character));
    }
    press(&mut app, KeyCode::Enter);

    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));
}

#[test]
fn shift_at_the_top_of_the_list_picks_upwards_instead_of_leaving_the_screen() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Up);

    assert_eq!(app.nav.at(), Screen::FINDINGS);
    assert_eq!(app.level, Level::List);
    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));
}

#[test]
fn the_bar_and_the_table_stay_inside_the_terminal_they_are_drawn_on() {
    for (width, height) in [(80u16, 24u16), (200, 30), (60, 12)] {
        let mut app = app();
        into(&mut app, Screen::FINDINGS, width, height);
        drawn_at(&app, width, height);
        shift(&mut app, KeyCode::Down);

        let page = drawn_at(&app, width, height);
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width}x{height}: {line}"
            );
        }
        assert!(page.contains("picked"), "{width}x{height}: {page}");
    }
}

#[test]
fn nothing_is_picked_on_a_screen_that_is_not_the_findings() {
    let mut app = app();
    into(
        &mut app,
        Screen::parse("ports").expect("a section"),
        200,
        30,
    );

    shift(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('x'));

    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}
