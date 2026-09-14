use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn_at, into, press, typed};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

#[test]
fn the_ports_screen_hides_and_shows_kinds_of_socket() {
    let mut app = app();
    into(&mut app, screen("ports"), 120, 24);

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
    press(&mut app, super::harness::number(screen("ports")));
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
    into(&mut app, screen("ports"), 120, 24);

    press(&mut app, KeyCode::Char('/'));
    typed(&mut app, "nginx");
    press(&mut app, KeyCode::Enter);
    assert!(app.panes().expect("a section").search().holding_back());

    press(&mut app, super::harness::number(screen("accounts")));
    press(&mut app, super::harness::number(screen("ports")));
    press(&mut app, KeyCode::Right);

    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert!(
        !app.panes().expect("a section").search().holding_back(),
        "the other view is showing everything"
    );
    let page = drawn_at(&app, 120, 24);
    assert!(
        page.contains("sshd") && page.contains("dockerd"),
        "the search belonged to the list it was typed into, so this one still stands over \
         every program the agent read: {page}"
    );
    assert!(
        page.contains("owner not resolved (7)"),
        "and over the sockets it could not name, counted rather than dropped: {page}"
    );
}

#[test]
fn a_letter_that_belongs_to_one_screen_does_nothing_on_another() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 120, 24);

    press(&mut app, KeyCode::Char('t'));

    assert!(
        !app.nav
            .lists
            .of("ports")
            .expect("the ports section")
            .hiding()
    );
}
