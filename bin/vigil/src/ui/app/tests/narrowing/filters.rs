use ratatui::crossterm::event::KeyCode;

use crate::ui::Screen;
use crate::ui::app::tests::harness::{app, drawn, drawn_at, into, press, typed};
use crate::ui::fixture::screen;

#[test]
fn the_same_key_narrows_a_list_of_findings_and_a_list_of_a_reading() {
    let mut app = app();
    into(&mut app, screen("ports"), 120, 24);

    press(&mut app, KeyCode::Char('f'));

    assert_eq!(app.nav.at(), screen("ports"));
    let page = drawn_at(&app, 120, 24);
    assert!(
        page.contains("\u{25b8} every kind") && page.contains("only tcp"),
        "f opened nothing here, and a key that works on one list and refuses on the next is \
         a key a reader stops trusting: {page}"
    );
}

#[test]
fn a_screen_that_is_one_page_has_nothing_to_narrow_and_says_so() {
    let mut app = app();
    into(&mut app, Screen::SUMMARY, 120, 24);

    press(&mut app, KeyCode::Char('f'));

    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("Nothing to filter on this screen"), "{page}");
    assert!(
        !page.contains("every kind"),
        "no empty choice was opened: {page}"
    );
}

#[test]
fn a_screen_that_is_one_page_and_not_a_list_says_there_is_nothing_to_put_in_an_order() {
    let mut app = app();
    into(&mut app, Screen::SUMMARY, 120, 24);

    press(&mut app, KeyCode::Char('s'));

    assert_eq!(app.nav.at(), Screen::SUMMARY);
    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("Nothing on this screen sorts"), "{page}");
}

#[test]
fn the_ports_and_the_accounts_have_a_search_of_their_own() {
    let mut app = app();
    into(&mut app, screen("ports"), 120, 24);

    press(&mut app, KeyCode::Char('/'));
    assert_eq!(
        app.nav.at(),
        screen("ports"),
        "it stays on the screen it was on"
    );
    typed(&mut app, "nginx q");
    assert!(!app.leaving, "the q was a letter, not the quit key");
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("nginx"), "{page}");
    assert!(!page.contains("/tmp/.x/nc"), "{page}");

    press(&mut app, KeyCode::Esc);
    assert!(!app.panes().expect("a section").search().holding_back());

    into(&mut app, screen("accounts"), 120, 40);
    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "contractor");
    press(&mut app, KeyCode::Enter);
    assert!(
        app.nav
            .lists
            .of("accounts")
            .expect("the accounts section")
            .search()
            .holding_back()
    );
    assert!(
        !app.nav
            .lists
            .of("ports")
            .expect("the ports section")
            .search()
            .holding_back(),
        "a search typed into one section is not a search in another"
    );
    assert!(drawn_at(&app, 120, 40).contains("contractor"));
}

#[test]
fn the_severity_floor_is_one_of_the_filters_now_and_is_reached_through_f() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::FINDINGS));

    press(&mut app, KeyCode::Char('f'));
    let choosing = drawn(&app);
    assert!(choosing.contains("show only"), "{choosing}");
    assert!(choosing.contains("critical and above"), "{choosing}");

    for _ in 0..4 {
        press(&mut app, KeyCode::Right);
    }
    press(&mut app, KeyCode::Enter);

    assert!(app.filter.holding_back(), "the floor is set");
    assert!(
        drawn(&app).contains("critical and above"),
        "and the list says what it is holding back: {}",
        drawn(&app)
    );
}

#[test]
fn a_choice_a_reader_left_alone_changes_nothing_at_all() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::FINDINGS));
    let before = drawn(&app);

    press(&mut app, KeyCode::Char('f'));
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Esc);

    assert!(!app.filter.holding_back());
    assert_eq!(drawn(&app), before, "Esc puts it back the way it was");
}

#[test]
fn a_search_belongs_to_the_list_it_was_typed_into_and_the_others_say_they_are_narrowed() {
    let mut app = app();
    press(&mut app, super::harness::number(screen("accounts")));
    press(&mut app, KeyCode::Right);
    assert_eq!(app.panes().expect("a section").showing(), 1);

    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "wheel");
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("wheel"), "{page}");
    assert_eq!(
        app.pane_keys(),
        vec!["group|wheel".to_string()],
        "the search narrowed the list it was typed into"
    );

    press(&mut app, super::harness::number(screen("ports")));
    press(&mut app, super::harness::number(screen("accounts")));
    press(&mut app, KeyCode::Left);
    assert_eq!(app.panes().expect("a section").showing(), 0);

    let page = drawn_at(&app, 160, 40);
    assert!(
        page.contains("contractor"),
        "the other list is whole: {page}"
    );
    assert!(
        page.contains("other list(s) narrowed by a search"),
        "and it says that one of them is not: {page}"
    );
}
