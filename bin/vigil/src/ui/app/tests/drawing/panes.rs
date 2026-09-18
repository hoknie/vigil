use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::app::graph::GRAPH;
use crate::ui::app::history::HISTORY;
use crate::ui::app::narrowing::DETAILS;
use crate::ui::app::tests::harness::{app, drawn_at, into, number, press};
use crate::ui::fixture::{self, screen};
use crate::ui::{Level, Screen};

const SIDES: [char; 6] = [
    '\u{2502}', '\u{2503}', '\u{256d}', '\u{256e}', '\u{250f}', '\u{2513}',
];

fn on_a_row_of(name: &str, pane: usize, key: &str, width: u16, height: u16) -> App {
    let mut app = app();
    app.view = fixture::view_with_launches();
    press(&mut app, number(screen(name)));
    drawn_at(&app, width, height);
    for _ in 0..pane {
        press(&mut app, KeyCode::Right);
    }
    for _ in 0..10 {
        if app.level == Level::List {
            break;
        }
        press(&mut app, KeyCode::Down);
    }
    for _ in 0..40 {
        if app
            .pane_row_under_the_cursor()
            .is_some_and(|row| row.key == key)
        {
            return app;
        }
        press(&mut app, KeyCode::Down);
    }
    panic!("{key} is not on the list of {name}");
}

fn every_kind_of_panes(width: u16, height: u16) -> Vec<(&'static str, App)> {
    let mut beside = app();
    into(&mut beside, Screen::FINDINGS, width, height);
    press(&mut beside, KeyCode::Enter);
    press(&mut beside, KeyCode::Enter);

    let mut sections = app();
    press(&mut sections, KeyCode::Char(DETAILS));

    let mut history = on_a_row_of(
        "programs",
        1,
        "run|alice|/usr/bin/nc.openbsd",
        width,
        height,
    );
    press(&mut history, KeyCode::Char(HISTORY));

    let mut graph = on_a_row_of("firewall", 2, "fw-interface|eth0", width, height);
    press(&mut graph, KeyCode::Char(GRAPH));

    vec![
        ("a list and its detail", beside),
        ("the sections and the one selected", sections),
        ("the history of a row", history),
        ("the path of a packet", graph),
    ]
}

#[test]
fn panes_with_frames_of_their_own_leave_one_border_on_each_side_and_one_thick_frame() {
    for (width, height) in [(80u16, 24u16), (120, 30)] {
        for (what, app) in every_kind_of_panes(width, height) {
            let page = drawn_at(&app, width, height);

            assert!(
                page.lines()
                    .nth(1)
                    .is_some_and(|line| line.starts_with(' ') && line.contains("as of 09:00:01")),
                "{what} at {width}x{height}: the section and the time of its reading are one \
                 plain line over the panes: {page}"
            );
            for line in page.lines() {
                let mut edge = line.chars().take(2);
                let (first, second) = (edge.next(), edge.next());
                assert!(
                    !(first.is_some_and(|it| SIDES.contains(&it))
                        && second.is_some_and(|it| SIDES.contains(&it))),
                    "{what} at {width}x{height}: a frame inside a frame puts two borders on the \
                     left, and each costs a column of content: {page}"
                );
                assert!(
                    line.chars().count() <= width as usize,
                    "{what} at {width}x{height}: {line}"
                );
            }
            assert_eq!(
                page.matches('\u{250f}').count(),
                1,
                "{what} at {width}x{height}: one thick frame, or the reader cannot tell where \
                 the arrows are: {page}"
            );
            assert!(
                page.lines()
                    .nth(2)
                    .is_some_and(|line| line.chars().count() == width as usize),
                "{what} at {width}x{height}: the frames of the panes reach both edges of the \
                 terminal, so the columns the outer frame took are the content's: {page}"
            );
        }
    }
}

#[test]
fn a_section_with_no_panes_of_its_own_keeps_its_one_thick_frame_and_no_heading_line() {
    let mut app = app();
    into(&mut app, screen("network"), 80, 24);

    let page = drawn_at(&app, 80, 24);

    assert!(
        page.lines()
            .nth(1)
            .is_some_and(|line| line.starts_with("\u{250f} \u{25b8} What is listening")),
        "a lone list is the panel of its section, named in its own edge: {page}"
    );
    assert_eq!(page.matches('\u{250f}').count(), 1, "{page}");
}
