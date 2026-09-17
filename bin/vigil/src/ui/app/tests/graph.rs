use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::graph::{GRAPH, WATCHING};
use crate::ui::app::history::HISTORY;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;
use crate::ui::{Level, Screen};

const INTERFACES: usize = 2;

const ETH0: &str = "fw-interface|eth0";

fn on_the_interfaces() -> App {
    let mut app = app();
    press(&mut app, number(screen("firewall")));
    drawn_at(&app, 120, 30);
    for _ in 0..INTERFACES {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    app
}

fn on_eth0() -> App {
    let mut app = on_the_interfaces();
    for _ in 0..20 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == ETH0)
        {
            return app;
        }
        press(&mut app, KeyCode::Down);
    }
    panic!("{ETH0} is not on the list: {:?}", app.pane_keys());
}

#[test]
fn p_draws_the_path_of_the_interface_under_the_cursor_and_esc_comes_back_to_the_list() {
    let mut app = on_eth0();

    press(&mut app, KeyCode::Char(GRAPH));
    let page = drawn_at(&app, 120, 40);

    assert!(app.graph.is_some(), "P drew nothing on {ETH0}");
    assert!(page.contains("HOW A PACKET TRAVELS"), "{page}");
    assert!(
        page.contains("eth0") && page.contains("192.168.1.23"),
        "{page}"
    );

    press(&mut app, KeyCode::Esc);
    drawn_at(&app, 120, 40);

    assert!(app.graph.is_none(), "Esc left the panel open");
}

#[test]
fn w_starts_the_counting_and_leaving_the_panel_stops_it() {
    let mut app = on_eth0();
    press(&mut app, KeyCode::Char(GRAPH));

    press(&mut app, KeyCode::Char(WATCHING));
    assert!(
        app.graph.as_ref().is_some_and(crate::ui::Graph::watching),
        "w did not start the counting"
    );

    press(&mut app, KeyCode::Esc);
    press(&mut app, KeyCode::Char(GRAPH));

    assert!(
        app.graph.as_ref().is_some_and(|graph| !graph.watching()),
        "a panel reopened still counting is a panel that never stopped, and the samples it \
         kept while nobody was looking are samples nobody asked for"
    );
}

#[test]
fn the_key_says_so_on_a_list_that_draws_a_path_and_says_nothing_on_one_that_does_not() {
    let interfaces = drawn_at(&on_the_interfaces(), 120, 30);
    assert!(interfaces.contains("P path"), "{interfaces}");

    let mut elsewhere = app();
    press(&mut elsewhere, number(screen("ports")));
    drawn_at(&elsewhere, 120, 30);
    while elsewhere.level != Level::List {
        press(&mut elsewhere, KeyCode::Down);
    }
    let ports = drawn_at(&elsewhere, 120, 30);

    assert!(
        !ports.contains("P path"),
        "a key drawn on a list where it does nothing is a key a reader learns to ignore: \
         {ports}"
    );
}

#[test]
fn p_on_a_list_that_draws_no_path_says_where_the_key_works_instead_of_doing_nothing() {
    let mut app = app();
    press(&mut app, number(screen("ports")));
    drawn_at(&app, 120, 30);
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }

    press(&mut app, KeyCode::Char(GRAPH));
    let page = drawn_at(&app, 120, 30);

    assert!(app.graph.is_none());
    assert!(page.contains("draws no graph of its rows"), "{page}");
}

#[test]
fn the_graph_and_the_history_are_two_panels_and_neither_opens_the_other() {
    let mut app = on_eth0();

    press(&mut app, KeyCode::Char(HISTORY));

    assert!(app.graph.is_none() && app.history.is_none());
    assert_eq!(
        app.nav.at(),
        screen("firewall"),
        "a key that belongs to another list must leave this one where it was"
    );
    assert_ne!(Screen::HOME, screen("firewall"));
}
