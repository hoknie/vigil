use ratatui::crossterm::event::KeyCode;

use crate::ui::app::tests::harness::{app, drawn, drawn_at, into, press};
use crate::ui::fixture::screen;
use crate::ui::{Anchor, Level, Reading, Screen, View};

#[test]
fn o_on_a_finding_about_a_socket_opens_the_ports_section_on_that_socket() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("network"));
    assert_eq!(
        app.level,
        Level::List,
        "straight onto the row: a cursor they cannot see is not being shown the object"
    );
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "tcp|0.0.0.0:4444",
        "and on the row the finding is about, not on the first one"
    );
}

#[test]
fn a_jump_asks_for_the_reading_that_holds_the_object_because_the_findings_screen_asks_for_none() {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 80, 30);

    assert_eq!(
        app.wanted_reading(),
        None,
        "the findings screen asks the daemon for no reading at all, which is the whole reason \
         a jump cannot assume one is already here"
    );

    app.view = View::nothing_yet("/nonexistent/vigil.sock");
    let anchor = Anchor {
        screen: screen("network"),
        key: "tcp|0.0.0.0:4444".into(),
    };

    assert_eq!(
        app.reading_needed(&anchor),
        Some("ports".to_string()),
        "with no reading held, the jump has to fetch the one that holds the object"
    );

    app.view = crate::ui::fixture::view();
    assert!(
        !matches!(app.view.reading("ports"), Reading::Unknown),
        "the fixture holds that reading"
    );
    assert_eq!(
        app.reading_needed(&anchor),
        None,
        "and a reading already held is not asked for twice"
    );
}

#[test]
fn o_on_a_finding_about_an_account_opens_the_accounts_section_at_that_account() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "user|account|contractor".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("accounts"));
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "account|contractor"
    );
    assert_eq!(app.level, Level::List);
    assert!(drawn(&app).contains("contractor"), "{}", drawn(&app));
}

#[test]
fn o_on_a_finding_about_a_group_opens_the_list_that_group_is_a_row_of() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "user|group|wheel".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("accounts"));
    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "group|wheel"
    );
}

#[test]
fn o_on_a_finding_about_a_program_opens_the_programs_section_on_the_running_list() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "process|exec|/tmp/.x/nc|www-data".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("programs"));
    assert_eq!(app.panes().expect("a section").showing(), 0);
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "exec|/tmp/.x/nc|www-data"
    );
}

#[test]
fn o_on_a_finding_about_a_cron_job_opens_the_startup_section_on_the_cron_list() {
    let mut app = app();
    app.view.found.findings[0].finding_key =
        "persistence|cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("startup"));
    assert_eq!(app.panes().expect("a section").showing(), 2);
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant"
    );
}

#[test]
fn a_finding_about_a_launch_walks_to_the_whole_key_because_that_family_adds_no_prefix() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    app.view.found.findings[0].finding_key = "run|alice|/usr/bin/nc.openbsd".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("programs"));
    assert_eq!(app.panes().expect("a section").showing(), 1);
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "run|alice|/usr/bin/nc.openbsd"
    );
}

#[test]
fn the_finding_about_a_dropping_spool_walks_to_the_row_that_says_so() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    app.view.found.findings[0].finding_key = "agent.buffer|launches".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), screen("programs"));
    assert_eq!(
        app.pane_keys()[app.panes().expect("a section").at()],
        "launches|dropping"
    );
}

#[test]
fn the_finding_about_what_is_waiting_for_a_receiver_walks_to_the_screen_that_names_it() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "agent.buffer|ndjson".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(
        app.nav.at(),
        Screen::SUMMARY,
        "a buffer is the agent's own, and the screen about this agent is where it is read"
    );
    let page = drawn_at(&app, 80, 60);
    assert!(page.contains("WHERE FINDINGS GO"), "{page}");
    assert!(
        page.lines()
            .any(|line| line.contains("ndjson") && line.contains("of 500")),
        "the row about that receiver says what is waiting for it and out of what: {page}"
    );
    assert!(
        !page.contains("draws no row"),
        "the screen carries that receiver, so there is nothing to apologise for: {page}"
    );
}

#[test]
fn a_receiver_this_agent_says_nothing_about_opens_a_screen_that_says_so_in_words() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "agent.buffer|webhook".into();
    into(&mut app, Screen::FINDINGS, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::SUMMARY);
    let page = drawn(&app);
    assert!(
        page.contains("draws no row for what is waiting to send"),
        "{page}"
    );
    assert!(
        page.contains("waiting for the webhook receiver"),
        "and which receiver it was about: {page}"
    );
    assert!(
        page.contains("09:00:00"),
        "and when the agent last had it in front of it: {page}"
    );
}

#[test]
fn the_two_buffers_a_finding_can_be_about_do_not_walk_to_the_same_place() {
    let mut spool = app();
    spool.view = crate::ui::fixture::view_with_launches();
    spool.view.found.findings[0].finding_key = "agent.buffer|launches".into();
    into(&mut spool, Screen::FINDINGS, 80, 30);
    press(&mut spool, KeyCode::Char('o'));

    let mut sending = app();
    sending.view.found.findings[0].finding_key = "agent.buffer|ndjson".into();
    into(&mut sending, Screen::FINDINGS, 80, 30);
    press(&mut sending, KeyCode::Char('o'));

    assert_eq!(spool.nav.at(), screen("programs"));
    assert_eq!(sending.nav.at(), Screen::SUMMARY);
    assert_ne!(
        spool.nav.at(),
        sending.nav.at(),
        "one key names the spool a plugin writes and the other what is waiting for a \
         receiver, and a reader sent to the wrong one reads about the wrong thing"
    );
}
