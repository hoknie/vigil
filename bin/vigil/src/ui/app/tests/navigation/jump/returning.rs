use ratatui::crossterm::event::KeyCode;

use crate::ui::Screen;
use crate::ui::app::tests::harness::{app, drawn, into, number, press};
use crate::ui::fixture::screen;

#[test]
fn a_jump_to_an_object_that_is_no_longer_in_the_reading_says_so_instead_of_moving() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "port.listen|tcp|0.0.0.0:9999".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::FINDINGS, "nothing moved");
    assert!(drawn(&app).contains("gone since"), "{}", drawn(&app));
}

#[test]
fn escape_after_o_comes_back_to_the_finding_it_was_pressed_on() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    let left_on = app
        .selected_finding()
        .expect("a finding under the cursor")
        .event_id
        .clone();
    app.view.found.findings[2].finding_key = "port.listen|tcp|0.0.0.0:4444".into();

    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.nav.at(), screen("ports"));

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::FINDINGS);
    assert_eq!(
        app.selected_finding().expect("back on a finding").event_id,
        left_on,
        "back to the row it was pressed on, not to the top of the list"
    );
}

#[test]
fn a_jump_remembers_one_place_and_forgets_it_once_it_has_been_used() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::FINDINGS);

    press(&mut app, KeyCode::Esc);

    assert_eq!(
        app.nav.at(),
        Screen::HOME,
        "the second Escape from the same rung is the ladder, not the ring"
    );
}

#[test]
fn walking_into_a_section_from_the_main_screen_leaves_nothing_for_escape_to_come_back_to() {
    let mut app = app();

    press(&mut app, number(screen("ports")));
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn changing_section_by_its_number_forgets_where_the_jump_came_from() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);
    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.nav.at(), screen("ports"));

    press(&mut app, number(screen("accounts")));
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn the_finding_that_is_no_longer_in_the_list_is_named_rather_than_returned_to_silently() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));
    app.view.found.findings.clear();
    app.settle();
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::FINDINGS, "back where it came from");
    let page = drawn(&app);
    assert!(page.contains("no longer in the list"), "{page}");
    assert!(page.contains("500"), "and how many the agent keeps: {page}");
}

#[test]
fn while_a_jump_is_armed_the_hint_names_the_finding_rather_than_the_main_screen() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    let page = drawn(&app);
    assert!(page.contains("back to the finding"), "{page}");
}
