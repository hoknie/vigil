use ratatui::crossterm::event::KeyCode;

use crate::ui::{Level, Screen};

use super::harness::{app, drawn_at, into, press};

#[test]
fn back_on_the_list_the_detail_follows_the_cursor_down_it() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 30);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Esc);

    press(&mut app, KeyCode::Down);

    assert_eq!(app.nav.findings.at(), 1);
    let page = drawn_at(&app, 200, 30);
    assert!(
        page.contains("LOW  A user logged in from a new address"),
        "the detail is showing the finding the cursor is now on: {page}"
    );
}

#[test]
fn every_motion_key_reaches_the_end_of_a_long_detail() {
    for key in [
        KeyCode::Down,
        KeyCode::Char('j'),
        KeyCode::PageDown,
        KeyCode::End,
        KeyCode::Char('G'),
    ] {
        let mut app = app();
        app.view.found.findings[0].before = Some(serde_json::json!({
            "process": {"cmdline": "a long one", "exe": "/usr/sbin/nginx"},
            "user": "root",
        }));
        app.view.found.findings[0].after = Some(serde_json::json!({
            "process": {"cmdline": "another", "exe": "/tmp/.x/nc"},
            "user": "www-data",
        }));
        into(&mut app, Screen::Findings, 200, 14);
        press(&mut app, KeyCode::Enter);

        let area = app.detail_area().expect("the detail is open");
        let total = app.detail_height(area);
        let last = total.saturating_sub(area.height as usize);
        assert!(last > 0, "the fixture has to be taller than the panel");

        for _ in 0..total {
            press(&mut app, key);
        }

        assert_eq!(
            app.nav.difference.top(),
            last,
            "{key:?} could not reach the end of the detail"
        );
    }
}

#[test]
fn a_detail_with_nothing_to_scroll_says_so_rather_than_looking_broken() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 40);
    press(&mut app, KeyCode::Enter);

    let tall = drawn_at(&app, 200, 40);
    assert!(tall.contains("all ") && tall.contains(" lines"), "{tall}");

    let short = drawn_at(&app, 200, 14);
    assert!(
        short.contains("lines 1-") && short.contains(" of "),
        "{short}"
    );
}

#[test]
fn on_a_terminal_with_room_for_one_half_the_detail_is_shown_only_while_it_has_the_arrows() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 24);

    press(&mut app, KeyCode::Enter);
    let detail = drawn_at(&app, 80, 24);
    assert!(detail.contains("CHANGED"), "{detail}");
    assert!(
        !detail.contains("A user logged in"),
        "no room for both: {detail}"
    );

    press(&mut app, KeyCode::Esc);
    let list = drawn_at(&app, 80, 24);
    assert_eq!(app.level, Level::List);
    assert!(!list.contains("CHANGED"), "{list}");
    assert!(list.contains("A user logged in"), "{list}");
}

#[test]
fn the_ports_screen_has_a_detail_of_its_own() {
    let mut app = app();
    into(&mut app, Screen::Ports, 200, 24);

    press(&mut app, KeyCode::Right);

    assert_eq!(app.level, Level::Detail);
    let page = drawn_at(&app, 200, 24);
    assert!(page.contains("THE SELECTED SOCKET"), "{page}");
    assert!(page.contains("/usr/sbin/nginx"), "the whole path: {page}");
    assert!(page.contains("suppressions:"), "{page}");
}
