use ratatui::crossterm::event::KeyCode;
use vigil_model::KillTarget;

use super::harness::{app, drawn_at, into, number, press};
use crate::ui::app::App;
use crate::ui::fixture::screen;
use crate::ui::{Choosing, Level};

const RUNNING: usize = 0;

const LAUNCHES: usize = 1;

const NC: &str = "run|alice|/usr/bin/nc.openbsd";

fn on_the_programs(pane: usize) -> App {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    press(&mut app, number(screen("programs")));
    drawn_at(&app, 120, 30);
    for _ in 0..pane {
        press(&mut app, KeyCode::Right);
    }
    while app.level != Level::List {
        press(&mut app, KeyCode::Down);
    }
    drawn_at(&app, 120, 30);
    app
}

fn cursor_to(app: &mut App, key: &str) {
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == key)
        {
            return;
        }
        press(app, KeyCode::Down);
    }
    panic!("{key} is not on the list: {:?}", app.pane_keys());
}

fn choose(app: &mut App, wanted: &str) {
    let Some(at) = app
        .chooser
        .offered()
        .iter()
        .position(|option| option == wanted)
    else {
        panic!("{wanted} is not offered: {:?}", app.chooser.offered());
    };
    while app.chooser.at() != at {
        press(app, KeyCode::Right);
    }
    press(app, KeyCode::Enter);
}

#[test]
fn a_running_program_is_marked_and_stopped_by_the_keys_that_close_a_socket() {
    let mut app = on_the_programs(RUNNING);
    cursor_to(&mut app, "exec|/tmp/.x/nc|www-data");

    press(&mut app, KeyCode::Char('x'));
    assert_eq!(app.marked(), vec!["exec|/tmp/.x/nc|www-data".to_string()]);

    press(&mut app, KeyCode::Char('K'));
    assert_eq!(
        app.chooser.choosing(),
        Some(Choosing::Kill(KillTarget::Program))
    );
    assert_eq!(
        app.chooser.keys(),
        &['S', 'K'],
        "a program has no socket to close, so the band has no D on it"
    );
    assert!(
        drawn_at(&app, 120, 30).contains("stop them by"),
        "{}",
        drawn_at(&app, 120, 30)
    );

    press(&mut app, KeyCode::Char('C'));
    assert_eq!(
        app.chooser.choosing(),
        None,
        "C walks away and asks nothing"
    );
}

#[test]
fn the_list_of_running_programs_says_at_the_bottom_that_k_stops_them() {
    let mut app = on_the_programs(RUNNING);
    cursor_to(&mut app, "exec|/tmp/.x/nc|www-data");

    let page = drawn_at(&app, 160, 30);

    assert!(page.contains("K stop"), "{page}");
}

#[test]
fn the_sockets_still_offer_all_three_ways_and_say_they_close_them() {
    let mut app = app();
    into(&mut app, screen("ports"), 120, 30);

    press(&mut app, KeyCode::Char('K'));

    assert_eq!(
        app.chooser.choosing(),
        Some(Choosing::Kill(KillTarget::Socket))
    );
    assert_eq!(app.chooser.keys(), &['S', 'K', 'D']);
}

#[test]
fn the_launches_are_not_a_list_the_console_acts_on() {
    let mut app = on_the_programs(LAUNCHES);
    cursor_to(&mut app, NC);

    press(&mut app, KeyCode::Char('K'));

    assert_eq!(
        app.chooser.choosing(),
        None,
        "a launch is a record of something that ran, and there is nothing in it to stop"
    );
}

#[test]
fn a_launch_list_narrows_to_the_person_under_the_cursor_and_widens_again() {
    let mut app = on_the_programs(LAUNCHES);
    cursor_to(&mut app, NC);

    press(&mut app, KeyCode::Char('f'));
    assert!(
        app.chooser
            .offered()
            .iter()
            .any(|option| option == "only program /usr/bin/nc.openbsd"),
        "{:?}",
        app.chooser.offered()
    );
    choose(&mut app, "only user alice");

    assert_eq!(app.pane_keys(), vec![NC.to_string()]);
    assert!(
        drawn_at(&app, 160, 30).contains("only user alice"),
        "{}",
        drawn_at(&app, 160, 30)
    );

    press(&mut app, KeyCode::Char('f'));
    choose(&mut app, "any user");
    assert!(app.pane_keys().len() > 1, "{:?}", app.pane_keys());
}

#[test]
fn a_launch_list_narrowed_to_a_person_and_then_a_program_holds_what_both_name() {
    let mut app = on_the_programs(LAUNCHES);
    cursor_to(&mut app, "run|root|/usr/bin/id");

    press(&mut app, KeyCode::Char('f'));
    choose(&mut app, "only user root");
    assert_eq!(app.pane_keys().len(), 2, "{:?}", app.pane_keys());

    cursor_to(&mut app, "run|root|/usr/bin/id");
    press(&mut app, KeyCode::Char('f'));
    choose(&mut app, "only program /usr/bin/id");
    assert_eq!(app.pane_keys(), vec!["run|root|/usr/bin/id".to_string()]);

    press(&mut app, KeyCode::Char('f'));
    choose(&mut app, "everything");
    assert!(app.pane_keys().len() > 2, "{:?}", app.pane_keys());
}

#[test]
fn a_launch_list_sorts_by_how_many_times_each_program_was_run() {
    let mut app = on_the_programs(LAUNCHES);

    press(&mut app, KeyCode::Char('s'));
    let runs: Vec<String> = app
        .chooser
        .offered()
        .iter()
        .filter(|option| option.contains("RUNS"))
        .cloned()
        .collect();
    assert_eq!(runs.len(), 2, "{:?}", app.chooser.offered());
    press(&mut app, KeyCode::Esc);

    press(&mut app, KeyCode::Char('s'));
    choose(&mut app, &runs[1]);

    let first_launch = app
        .pane_keys()
        .into_iter()
        .find(|key| key.starts_with("run|"));
    assert_eq!(first_launch.as_deref(), Some(NC), "{:?}", app.pane_keys());
    assert_eq!(
        app.sorted(),
        crate::ui::Sorting {
            by: 3,
            descending: true
        },
        "the order chosen is RUNS, largest first, and it belongs to this list"
    );
}
