use ratatui::crossterm::event::KeyCode;

use crate::ui::screens::ports;
use crate::ui::{Level, Screen, Subject};

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
    assert_eq!(app.nav.lists.ports.showing(), ports::Arrangement::ByProgram);
    assert!(grouped.contains("grouped by program"), "{grouped}");
    assert!(grouped.contains("nginx (2)"), "{grouped}");

    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('t'));
    let hidden = drawn_at(&app, 120, 24);
    assert!(!hidden.contains("0.0.0.0:4444"), "{hidden}");
    assert!(hidden.contains("hiding tcp"), "{hidden}");

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Left);
    assert_eq!(app.nav.lists.ports.showing(), ports::Arrangement::Flat);
    assert!(
        app.ports_protocols.holding_back(),
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
    assert!(app.nav.lists.ports.search().holding_back());

    press(&mut app, super::harness::number(Screen::Accounts));
    press(&mut app, super::harness::number(Screen::Ports));
    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.lists.ports.showing(), ports::Arrangement::ByProgram);
    assert!(
        !app.nav.lists.ports.search().holding_back(),
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

    assert!(!app.ports_protocols.holding_back());
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
fn the_severity_floor_says_where_it_lives_rather_than_moving_the_reader_to_it() {
    let mut app = app();
    into(&mut app, Screen::Ports, 120, 24);

    press(&mut app, KeyCode::Char('s'));

    assert_eq!(app.nav.at(), Screen::Ports);
    assert!(
        drawn_at(&app, 120, 24).contains("findings section"),
        "{}",
        drawn_at(&app, 120, 24)
    );
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
    assert!(!app.nav.lists.ports.search().holding_back());

    into(&mut app, Screen::Accounts, 120, 40);
    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "docker");
    press(&mut app, KeyCode::Enter);
    assert!(app.nav.lists.accounts.search().holding_back());
    assert!(!app.nav.lists.ports.search().holding_back());
    assert!(drawn_at(&app, 120, 40).contains("docker"));
}

#[test]
fn the_severity_floor_moves_in_both_directions_and_says_where_it_is() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Findings));

    press(&mut app, KeyCode::Char('s'));
    assert!(drawn(&app).contains("low and above"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('S'));
    assert!(
        !app.filter.holding_back(),
        "and comes back to showing everything"
    );
}

#[test]
fn a_search_belongs_to_the_list_it_was_typed_into_and_the_others_say_they_are_narrowed() {
    let mut app = app();
    press(&mut app, super::harness::number(Screen::Accounts));
    press(&mut app, KeyCode::Right);
    assert_eq!(app.nav.lists.accounts.showing(), Subject::Groups);

    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "docker");
    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 120, 40);
    assert!(page.contains("docker"), "{page}");
    assert!(!page.contains("wheel"), "{page}");

    press(&mut app, super::harness::number(Screen::Ports));
    press(&mut app, super::harness::number(Screen::Accounts));
    press(&mut app, KeyCode::Left);
    assert_eq!(app.nav.lists.accounts.showing(), Subject::Users);

    let page = drawn_at(&app, 160, 40);
    assert!(page.contains("backdoor"), "the other list is whole: {page}");
    assert!(
        page.contains("other list(s) narrowed by a search"),
        "and it says that one of them is not: {page}"
    );
}
