use ratatui::crossterm::event::KeyCode;

use crate::ui::Level;
use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, click, drawn_at, number, press, where_it_says};
use crate::ui::fixture::grouped::{DOCKER, HOST, PODMAN, SECTION, section};
use crate::ui::fixture::screen;
use crate::ui::sections::instead;

const WIDE: (u16, u16) = (120, 40);

fn on_the_engines() -> App {
    let mut app = app();
    press(&mut app, number(screen(SECTION)));
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

fn clicked(app: &mut App, word: &str) {
    let page = drawn_at(app, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, word);
    click(app, column + 1, row);
}

#[test]
fn a_click_on_the_first_row_switches_the_group_and_leaves_the_arrows_on_that_row() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();

    clicked(&mut app, "docker");

    assert_eq!(app.group_showing(), Some(DOCKER));
    assert_eq!(
        app.level,
        Level::Groups,
        "the reader pressed a group, so the keys carry on where the click landed"
    );
    assert_eq!(
        app.pane().expect("a list of the group").name(),
        "images",
        "and the first list of the group it switched to is the one under it"
    );
}

#[test]
fn a_click_on_the_second_row_switches_the_list_and_leaves_the_arrows_on_that_row() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    clicked(&mut app, "docker");

    clicked(&mut app, "volumes");

    assert_eq!(app.group_showing(), Some(DOCKER), "the group did not move");
    assert_eq!(app.pane().expect("a list of the group").name(), "volumes");
    assert_eq!(app.level, Level::Menu);
}

#[test]
fn a_click_on_a_group_with_nothing_to_show_says_so_rather_than_answering_nothing() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();

    clicked(&mut app, "podman");

    assert_eq!(app.group_showing(), Some(PODMAN));
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    assert!(page.contains("[podman]"), "{page}");
    assert!(
        page.contains(crate::ui::fixture::grouped::NOT_INSTALLED),
        "{page}"
    );
}

#[test]
fn every_group_and_every_list_a_click_reaches_is_reached_by_the_arrows_as_well() {
    let _standing = instead::drawn(SECTION, section);
    let mut app = on_the_engines();
    clicked(&mut app, "podman");
    let by_a_click = app.group_showing();

    let mut walking = on_the_engines();
    press(&mut walking, KeyCode::Left);

    assert_eq!(
        walking.group_showing(),
        by_a_click,
        "the mouse is a second way to what the keys already reach, and never the only way"
    );
    assert_eq!(walking.group_showing(), Some(PODMAN));
    press(&mut walking, KeyCode::Right);
    assert_eq!(walking.group_showing(), Some(HOST));
}
