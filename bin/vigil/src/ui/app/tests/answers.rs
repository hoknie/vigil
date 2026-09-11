use ratatui::crossterm::event::KeyCode;
use vigil_model::{CollectorRefusal, CollectorState, Request, Response, Snapshot};

use crate::link::{Trouble, TroubleKind};
use crate::ui::{Program, Reading, Screen, View};

use super::harness::{app, drawn, into, number, press};

fn answered(answer: Response) -> View {
    let app = app();
    let mut view = View::nothing_yet("/nonexistent/vigil.sock");
    app.read(
        &mut view,
        &Request::Snapshot {
            collector: "launches".into(),
        },
        answer,
    );
    view
}

#[test]
fn an_answer_with_no_reading_and_no_reason_is_a_reading_that_has_not_come_round_yet() {
    let view = answered(Response::Snapshot {
        collector: "launches".into(),
        snapshot: None,
        refusal: None,
    });

    assert!(matches!(view.reading("launches"), Reading::NotTakenYet));
}

#[test]
fn an_answer_with_no_reading_and_a_reason_is_a_refusal_and_never_a_reading_on_its_way() {
    let view = answered(Response::Snapshot {
        collector: "launches".into(),
        snapshot: None,
        refusal: Some(CollectorRefusal::new(
            CollectorState::Unavailable,
            "auditd is not running",
        )),
    });

    match view.reading("launches") {
        Reading::Refused(refusal) => {
            assert_eq!(refusal.state, Some(CollectorState::Unavailable));
            assert_eq!(refusal.reason, "auditd is not running");
        }
        _ => panic!("a collector that could not read was drawn as one that has not read yet"),
    }
}

#[test]
fn a_reading_that_arrived_with_a_reason_beside_it_is_still_the_reading() {
    let view = answered(Response::Snapshot {
        collector: "launches".into(),
        snapshot: Some(Snapshot::new("launches", "2026-09-09T09:00:00.000Z")),
        refusal: Some(CollectorRefusal::new(
            CollectorState::Degraded,
            "the audit rule is not loaded",
        )),
    });

    assert!(
        matches!(view.reading("launches"), Reading::Taken(_)),
        "what was read is shown; that it is partial belongs to the state on the summary"
    );
}

#[test]
fn r_asks_for_a_refresh_instead_of_performing_one() {
    let mut app = app();

    press(&mut app, KeyCode::Char('r'));

    assert!(app.refresh_wanted);
}

#[test]
fn the_main_screen_asks_for_no_reading_at_all() {
    let app = app();

    assert_eq!(
        app.wanted_reading(),
        None,
        "the counts and the states on it both come out of the status"
    );
}

#[test]
fn the_console_asks_for_the_reading_behind_the_section_it_is_showing_and_no_other() {
    let mut app = app();

    press(&mut app, number(Screen::Ports));
    assert_eq!(app.wanted_reading(), Some("ports"));

    press(&mut app, number(Screen::Accounts));
    assert_eq!(app.wanted_reading(), Some("users"));

    press(&mut app, number(Screen::Startup));
    assert_eq!(app.wanted_reading(), Some("persistence"));

    press(&mut app, number(Screen::Findings));
    assert_eq!(
        app.wanted_reading(),
        None,
        "the findings come from the ring, not from a collector"
    );
}

#[test]
fn the_two_lists_of_one_section_belong_to_two_collectors_and_only_the_open_one_is_asked_for() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();

    press(&mut app, number(Screen::Programs));
    assert_eq!(app.nav.lists.programs.showing(), Program::Running);
    assert_eq!(app.wanted_reading(), Some("processes"));

    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.lists.programs.showing(), Program::Launches);
    assert_eq!(app.wanted_reading(), Some("launches"));
}

#[test]
fn the_console_does_not_ask_for_the_reading_of_a_collector_that_is_switched_off() {
    let mut app = app();

    press(&mut app, number(Screen::Programs));
    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.lists.programs.showing(), Program::Launches);
    assert_eq!(
        app.wanted_reading(),
        None,
        "asking would be answered `unknown_collector`, and that would be drawn as a refusal"
    );
}

#[test]
fn opening_a_section_asks_for_its_reading_at_once_and_not_at_the_next_tick() {
    let mut app = app();
    app.refresh_wanted = false;

    press(&mut app, number(Screen::Startup));

    assert!(
        app.refresh_wanted,
        "a section opened on a reading nobody has asked for yet is an empty table"
    );
}

#[test]
fn walking_along_the_row_of_lists_asks_again_because_the_collector_changed() {
    let mut app = app();
    press(&mut app, number(Screen::Programs));
    app.refresh_wanted = false;

    press(&mut app, KeyCode::Right);

    assert!(app.refresh_wanted);
}

#[test]
fn with_nothing_ever_answered_the_page_says_so_instead_of_drawing_empty_tables() {
    let mut app = app();
    app.view = View::nothing_yet("/run/vigil/vigil.sock");
    app.view.trouble = Some(Trouble::new(
        "/run/vigil/vigil.sock",
        TroubleKind::Absent,
        "No such file or directory (os error 2)",
    ));

    let page = drawn(&app);

    assert!(page.contains("not answering"), "{page}");
    assert!(!page.contains("PROTO"), "{page}");
}

#[test]
fn an_agent_that_stops_answering_keeps_its_last_reading_on_screen_and_marks_it_old() {
    let mut app = app();
    into(&mut app, Screen::Ports, 80, 30);

    app.refresh();

    let page = drawn(&app);
    assert!(!app.answered(), "the last question was not answered");
    assert!(
        page.contains("0.0.0.0:4444"),
        "the rows are still there: {page}"
    );
    assert!(page.contains("NOT ANSWERING"), "and marked as old: {page}");
}

#[test]
fn refreshing_against_a_socket_that_is_not_there_is_a_screen_and_not_a_panic() {
    let mut app = app();

    app.refresh();

    assert!(!app.view.answered());
    assert!(app.view.trouble.is_some());
}
