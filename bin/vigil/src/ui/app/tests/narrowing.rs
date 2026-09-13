use ratatui::crossterm::event::KeyCode;

use crate::ui::{Level, Screen};

use super::harness::{app, drawn, drawn_at, into, press, typed};

#[test]
fn the_ports_screen_hides_and_shows_kinds_of_socket() {
    let mut app = app();
    into(&mut app, Screen::Ports, 120, 24);

    press(&mut app, KeyCode::Char('t'));
    let hidden = drawn_at(&app, 120, 24);
    assert!(
        !hidden.contains("0.0.0.0:4444"),
        "the tcp rows went: {hidden}"
    );
    assert!(hidden.contains("hiding tcp"), "{hidden}");

    press(&mut app, KeyCode::Char('a'));
    assert!(drawn_at(&app, 120, 24).contains("0.0.0.0:4444"));
}

#[test]
fn the_ports_views_are_a_submenu_and_the_filter_holds_across_both_of_them() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Ports));
    assert_eq!(app.level, Level::Menu);

    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("[sockets]"), "{page}");
    assert!(page.contains("by program"), "{page}");

    press(&mut app, KeyCode::Right);
    let grouped = drawn_at(&app, 120, 24);
    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert!(grouped.contains("grouped by program"), "{grouped}");
    assert!(grouped.contains("nginx (1)"), "{grouped}");

    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('t'));
    let hidden = drawn_at(&app, 120, 24);
    assert!(!hidden.contains("0.0.0.0:4444"), "{hidden}");
    assert!(hidden.contains("hiding tcp"), "{hidden}");

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.panes().expect("a section").showing(), 0);
    assert!(
        app.panes().expect("a section").hiding(),
        "the filter is the screen's, not the view's, and climbing a rung does not drop it"
    );
}

#[test]
fn a_search_on_one_ports_view_does_not_narrow_the_other() {
    let mut app = app();
    into(&mut app, Screen::Ports, 120, 24);

    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "nginx");
    press(&mut app, KeyCode::Enter);
    assert!(app.panes().expect("a section").search().holding_back());

    press(&mut app, super::harness::number(Screen::Accounts));
    press(&mut app, super::harness::number(Screen::Ports));
    press(&mut app, KeyCode::Right);

    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert!(
        !app.panes().expect("a section").search().holding_back(),
        "the other view is showing everything"
    );
    assert!(
        drawn_at(&app, 120, 24).contains("0.0.0.0:4444"),
        "{}",
        drawn_at(&app, 120, 24)
    );
}

#[test]
fn a_letter_that_belongs_to_one_screen_does_nothing_on_another() {
    let mut app = app();
    into(&mut app, Screen::Findings, 120, 24);

    press(&mut app, KeyCode::Char('t'));

    assert!(
        !app.nav
            .lists
            .of("ports")
            .expect("the ports section")
            .hiding()
    );
}

#[test]
fn a_search_narrows_the_findings_and_the_letters_do_not_reach_the_console() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Findings));

    press(&mut app, KeyCode::Char('/'));
    assert_eq!(
        app.level,
        Level::List,
        "the search takes the arrows into the list"
    );

    typed(&mut app, "logged in q");
    assert!(!app.leaving, "the q was a letter, not the quit key");

    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);

    let page = drawn(&app);
    assert!(page.contains("A user logged in"), "{page}");
    assert!(!page.contains("A new listening port"), "{page}");
}

#[test]
fn escape_takes_a_search_off_before_it_takes_the_screen_away() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Findings));
    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "nothing matches this");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::List);
    assert!(app.filter.holding_back());

    press(&mut app, KeyCode::Esc);
    assert!(!app.filter.holding_back(), "the search went first");
    assert_eq!(app.level, Level::List, "and the arrows stayed on the list");
    assert!(
        drawn(&app).contains("A new listening port"),
        "{}",
        drawn(&app)
    );

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::Home, "then out of the section");

    press(&mut app, KeyCode::Esc);
    assert!(!app.leaving, "and the main screen is the top");
}

#[test]
fn the_search_opens_on_the_screen_the_reader_is_on_and_never_moves_them() {
    for screen in [Screen::Ports, Screen::Accounts, Screen::Findings] {
        let mut app = app();
        press(&mut app, super::harness::number(screen));
        assert_ne!(
            app.level,
            Level::Detail,
            "the arrows start above the detail"
        );

        press(&mut app, KeyCode::Char('/'));

        assert_eq!(app.nav.at(), screen, "the screen changed under the reader");
        assert_eq!(
            app.level,
            Level::List,
            "and the arrows went into this screen's list"
        );
        assert!(app.typing(), "{} did not open its box", screen.name());
    }
}

#[test]
fn on_a_screen_with_no_list_the_search_says_so_rather_than_moving_the_reader() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Summary));

    press(&mut app, KeyCode::Char('/'));

    assert_eq!(app.nav.at(), Screen::Summary);
    assert!(!app.typing());
    assert!(
        drawn(&app).contains("Nothing to search on this screen"),
        "{}",
        drawn(&app)
    );
}

#[test]
fn the_main_screen_has_no_search_and_says_where_the_search_lives() {
    let mut app = app();

    press(&mut app, KeyCode::Char('/'));

    assert_eq!(
        app.nav.at(),
        Screen::Home,
        "and it does not move the reader"
    );
    assert!(!app.typing());
    assert!(
        drawn(&app).contains("open one and press /"),
        "{}",
        drawn(&app)
    );
}

#[test]
fn a_screen_with_nothing_to_filter_says_so_rather_than_opening_an_empty_choice() {
    let mut app = app();
    into(&mut app, Screen::Ports, 120, 24);

    press(&mut app, KeyCode::Char('f'));

    assert_eq!(app.nav.at(), Screen::Ports);
    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("Nothing to filter here"), "{page}");
    assert!(page.contains("findings"), "and where it does live: {page}");
    assert!(page.contains("Press / to search"), "{page}");
    assert!(
        !page.contains("show only"),
        "no empty choice was opened: {page}"
    );
}

#[test]
fn a_screen_that_is_one_page_and_not_a_list_says_there_is_nothing_to_put_in_an_order() {
    let mut app = app();
    into(&mut app, Screen::Summary, 120, 24);

    press(&mut app, KeyCode::Char('s'));

    assert_eq!(app.nav.at(), Screen::Summary);
    let page = drawn_at(&app, 120, 24);
    assert!(page.contains("Nothing on this screen sorts"), "{page}");
}

#[test]
fn the_ports_and_the_accounts_have_a_search_of_their_own() {
    let mut app = app();
    into(&mut app, Screen::Ports, 120, 24);

    press(&mut app, KeyCode::Char('/'));
    assert_eq!(
        app.nav.at(),
        Screen::Ports,
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

    into(&mut app, Screen::Accounts, 120, 40);
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
    press(&mut app, super::harness::number(Screen::Findings));

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
    press(&mut app, super::harness::number(Screen::Findings));
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
    press(&mut app, super::harness::number(Screen::Accounts));
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

    press(&mut app, super::harness::number(Screen::Ports));
    press(&mut app, super::harness::number(Screen::Accounts));
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
