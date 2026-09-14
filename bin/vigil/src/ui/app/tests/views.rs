use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;

use super::harness::{app, drawn_at, into, press};
use crate::ui::fixture::screen;

const WIDE: u16 = 140;

fn on_the_units() -> App {
    let mut app = app();
    into(&mut app, screen("startup"), WIDE, 30);
    app
}

fn onto(app: &mut App, name: &str) {
    for _ in 0..app.pane_keys().len() {
        if app.pane_keys()[app.panes().expect("a section").at()].contains(name) {
            return;
        }
        press(app, KeyCode::Down);
    }
    panic!("{name} is not a row of this list: {:?}", app.pane_keys());
}

fn cursor_row(app: &App) -> String {
    drawn_at(app, WIDE, 30)
        .lines()
        .find(|line| line.contains(" > "))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn the_key_switches_the_units_to_a_tree_and_back_again() {
    let mut app = on_the_units();
    assert!(drawn_at(&app, WIDE, 30).contains("[t list]"));

    press(&mut app, KeyCode::Char('t'));
    let tree = drawn_at(&app, WIDE, 30);
    assert!(tree.contains("[t tree]"), "{tree}");
    assert!(tree.contains("as written in the files"), "{tree}");

    press(&mut app, KeyCode::Char('t'));
    let list = drawn_at(&app, WIDE, 30);
    assert!(list.contains("[t list]"), "{list}");
    assert!(!list.contains("as written in the files"), "{list}");
}

#[test]
fn switching_the_view_leaves_the_reader_on_the_unit_they_were_on() {
    let mut app = on_the_units();
    onto(&mut app, "rescue-shell.service");
    let before = app.pane_keys()[app.panes().expect("a section").at()].clone();
    assert!(cursor_row(&app).contains("rescue-shell.service"));

    press(&mut app, KeyCode::Char('t'));

    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        before,
        "the cursor holds the key of its row and not its number: {}",
        drawn_at(&app, WIDE, 30)
    );
    assert!(
        cursor_row(&app).contains("rescue-shell.service"),
        "{}",
        drawn_at(&app, WIDE, 30)
    );
}

#[test]
fn the_key_says_where_it_lives_on_a_list_it_does_not_switch() {
    let mut app = app();
    press(&mut app, super::harness::number(screen("startup")));
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.panes().expect("a section").showing(), 1);

    press(&mut app, KeyCode::Char('t'));

    let page = drawn_at(&app, WIDE, 30);
    assert!(page.contains("belongs to the units list"), "{page}");
    assert!(!page.contains("[t tree]"), "{page}");
}

#[test]
fn the_detail_behind_a_row_is_the_same_unit_in_either_view() {
    let mut app = on_the_units();
    onto(&mut app, "nginx.service");
    press(&mut app, KeyCode::Right);
    let flat = drawn_at(&app, WIDE, 30);
    assert!(flat.contains("NGINX.SERVICE"), "{flat}");

    press(&mut app, KeyCode::Char('t'));

    let tree = drawn_at(&app, WIDE, 30);
    assert!(tree.contains("NGINX.SERVICE"), "{tree}");
    assert!(tree.contains("multi-user.target · WantedBy"), "{tree}");
}

#[test]
fn a_suppression_is_written_against_the_same_key_in_either_view() {
    let mut app = on_the_units();
    onto(&mut app, "nginx.service");
    press(&mut app, KeyCode::Char('t'));
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Char('G'));

    let page = drawn_at(&app, WIDE, 40);

    assert!(
        page.contains("persistence|unit|nginx.service"),
        "the key a suppression is written against is copied off the screen: {page}"
    );
}
