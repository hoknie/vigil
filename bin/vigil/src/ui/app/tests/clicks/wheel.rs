use ratatui::crossterm::event::KeyCode;

use crate::ui::Level;
use crate::ui::app::App;
use crate::ui::app::tests::harness::{app, drawn_at, into, press, wheel};
use crate::ui::fixture::screen;

const WIDE: (u16, u16) = (120, 20);

fn with_a_detail_beside_the_list() -> App {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

#[test]
fn the_wheel_over_a_list_walks_the_list_and_leaves_the_panel_where_it_was() {
    let mut app = with_a_detail_beside_the_list();
    let first = app.panes().map(|panes| panes.at());

    wheel(&mut app, 4, 8, true);
    wheel(&mut app, 4, 8, true);

    assert_ne!(
        app.panes().map(|panes| panes.at()),
        first,
        "the wheel over a list is the list scrolling, which on this console is the cursor \
         walking down it: {}",
        drawn_at(&app, WIDE.0, WIDE.1)
    );
    assert_eq!(
        app.nav.difference.top(),
        0,
        "and the panel beside it was not under the pointer, so it did not move"
    );
}

#[test]
fn the_wheel_over_the_panel_scrolls_the_panel_and_leaves_the_list_where_it_was() {
    let mut app = with_a_detail_beside_the_list();
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let row = app.panes().map(|panes| panes.at());

    for _ in 0..4 {
        wheel(&mut app, WIDE.0 - 6, 10, true);
    }

    assert!(
        app.nav.difference.top() > 0,
        "the panel under the pointer is the thing that scrolls: {page}"
    );
    assert_eq!(
        app.panes().map(|panes| panes.at()),
        row,
        "and the list under it kept its row"
    );

    let scrolled = app.nav.difference.top();
    for _ in 0..8 {
        wheel(&mut app, WIDE.0 - 6, 10, false);
    }
    assert!(
        app.nav.difference.top() < scrolled,
        "the wheel turns both ways"
    );
}

#[test]
fn the_wheel_over_a_list_the_arrows_had_left_gives_them_back_to_it() {
    let mut app = with_a_detail_beside_the_list();
    assert_eq!(app.level, Level::Detail, "the arrows are in the panel");

    wheel(&mut app, 4, 8, true);

    assert_eq!(
        app.level,
        Level::List,
        "the reader scrolled the list, so the list is where the keys are now, and the screen \
         says so"
    );
}
