use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::graph::GRAPH;
use crate::ui::app::history::HISTORY;
use crate::ui::app::tests::harness::{app, click, drawn_at, into, number, press, where_it_says};
use crate::ui::fixture::screen;
use crate::ui::{Level, Target};

const WIDE: (u16, u16) = (120, 40);

fn on_a_socket() -> App {
    let mut app = app();
    into(&mut app, screen("ports"), WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Enter);
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

fn on_a_launch() -> App {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    press(&mut app, number(screen("programs")));
    drawn_at(&app, WIDE.0, WIDE.1);
    press(&mut app, KeyCode::Right);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == "run|alice|/usr/bin/nc.openbsd")
        {
            break;
        }
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, WIDE.0, WIDE.1);
    app
}

#[test]
fn a_click_on_a_button_does_what_pressing_that_button_with_the_keys_does() {
    let mut clicked = on_a_socket();
    let page = drawn_at(&clicked, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "[ S suppress it ]");
    click(&mut clicked, column + 2, row);

    let mut pressed = on_a_socket();
    press(&mut pressed, KeyCode::Right);
    while pressed
        .acts()
        .button(pressed.button)
        .is_some_and(|button| button.key != 'S')
    {
        press(&mut pressed, KeyCode::Right);
    }
    press(&mut pressed, KeyCode::Enter);

    assert!(
        clicked.paper.is_some(),
        "the button drawn as [ S suppress it ] hands back the sheet the S key hands back: {page}"
    );
    assert_eq!(
        clicked.paper.as_ref().map(|sheet| sheet.caption.clone()),
        pressed.paper.as_ref().map(|sheet| sheet.caption.clone()),
        "a button pressed with the mouse must walk the same road as the key, or the two ways of \
         pressing it can drift apart"
    );
    assert_eq!(clicked.paper, pressed.paper);
}

#[test]
fn a_click_on_the_back_button_of_a_panel_leaves_it_the_way_escape_does() {
    let mut app = on_a_launch();
    press(&mut app, KeyCode::Char(HISTORY));
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    assert!(app.history.is_some(), "the history panel is open: {page}");

    let (column, row) = where_it_says(&page, "[ \u{2190} Back ]");
    click(&mut app, column + 3, row);

    assert!(
        app.history.is_none(),
        "the button the panel draws is the way out of it for a reader who reached it with the \
         mouse: {page}"
    );
}

#[test]
fn a_click_on_the_counting_button_of_the_path_panel_starts_and_stops_the_counting() {
    let mut app = app();
    press(&mut app, number(screen("firewall")));
    drawn_at(&app, WIDE.0, WIDE.1);
    for _ in 0..2 {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    for _ in 0..20 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == "fw-interface|eth0")
        {
            break;
        }
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Char(GRAPH));
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "[ w start counting ]");

    click(&mut app, column + 3, row);

    assert!(
        app.graph.as_ref().is_some_and(crate::ui::Graph::watching),
        "the button beside Back counts packets, and a click on it is the w key: {page}"
    );

    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "[ w stop counting ]");
    click(&mut app, column + 3, row);

    assert!(
        app.graph.as_ref().is_some_and(|graph| !graph.watching()),
        "and clicking it again stops it, the way the key does: {page}"
    );
}

#[test]
fn a_click_lands_on_what_is_drawn_on_top_and_not_on_what_it_covers() {
    let mut app = on_a_socket();
    press(&mut app, KeyCode::Char('K'));
    let page = drawn_at(&app, WIDE.0, WIDE.1);
    let (column, row) = where_it_says(&page, "stop the process now");

    let under = app.targets_under(column, row);
    assert!(
        under
            .iter()
            .any(|target| matches!(target, Target::Aim(crate::ui::Aim::Option(_)))),
        "the band is over that cell: {under:?}"
    );
    assert!(
        under.len() > 1,
        "the band was drawn over something, and the something is still recorded under it: \
         {under:?}"
    );
    assert!(
        under
            .first()
            .is_some_and(|target| matches!(target, Target::Aim(crate::ui::Aim::Option(_)))),
        "what the reader sees at a cell is what a click there presses, so the band that covers \
         the page wins over the page: {page}"
    );
}
