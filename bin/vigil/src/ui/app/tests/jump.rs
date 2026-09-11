use ratatui::crossterm::event::KeyCode;

use crate::ui::{Anchor, Level, Program, Reading, Screen, Startup, Subject, View};

use super::harness::{app, drawn, into, number, press};

#[test]
fn o_on_a_finding_about_a_socket_opens_the_ports_section_on_that_socket() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Ports);
    assert_eq!(
        app.level,
        Level::List,
        "straight onto the row: a cursor they cannot see is not being shown the object"
    );
    assert_eq!(
        app.ports_keys()[app.nav.lists.ports.at()],
        "tcp|0.0.0.0:4444",
        "and on the row the finding is about, not on the first one"
    );
}

#[test]
fn a_jump_asks_for_the_reading_that_holds_the_object_because_the_findings_screen_asks_for_none() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);

    assert_eq!(
        app.wanted_reading(),
        None,
        "the findings screen asks the daemon for no reading at all, which is the whole reason \
         a jump cannot assume one is already here"
    );

    app.view = View::nothing_yet("/nonexistent/vigil.sock");
    let anchor = Anchor {
        screen: Screen::Ports,
        key: "tcp|0.0.0.0:4444".into(),
    };

    assert_eq!(
        app.reading_needed(&anchor),
        Some("ports"),
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
    app.view.found.findings[0].finding_key = "user|account|backdoor".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Accounts);
    assert_eq!(
        app.accounts_keys()[app.nav.lists.accounts.at()],
        "account|backdoor"
    );
    assert_eq!(app.level, Level::List);
    assert!(drawn(&app).contains("backdoor"), "{}", drawn(&app));
}

#[test]
fn o_on_a_finding_about_a_group_opens_the_list_that_group_is_a_row_of() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "user|group|docker".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Accounts);
    assert_eq!(app.nav.lists.accounts.showing(), Subject::Groups);
    assert_eq!(
        app.accounts_keys()[app.nav.lists.accounts.at()],
        "group|docker"
    );
}

#[test]
fn o_on_a_finding_about_a_program_opens_the_programs_section_on_the_running_list() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "process|exec|/tmp/.x/nc|www-data".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Programs);
    assert_eq!(app.nav.lists.programs.showing(), Program::Running);
    assert_eq!(
        app.programs_keys()[app.nav.lists.programs.at()],
        "exec|/tmp/.x/nc|www-data"
    );
}

#[test]
fn o_on_a_finding_about_a_cron_job_opens_the_startup_section_on_the_cron_list() {
    let mut app = app();
    app.view.found.findings[0].finding_key =
        "persistence|cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Startup);
    assert_eq!(app.nav.lists.startup.showing(), Startup::Cron);
    assert_eq!(
        app.startup_keys()[app.nav.lists.startup.at()],
        "cron|/var/spool/cron/crontabs/www-data|www-data|/tmp/.x/implant"
    );
}

#[test]
fn a_finding_about_a_launch_walks_to_the_whole_key_because_that_family_adds_no_prefix() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    app.view.found.findings[0].finding_key = "run|alice|/usr/bin/nmap".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Programs);
    assert_eq!(app.nav.lists.programs.showing(), Program::Launches);
    assert_eq!(
        app.programs_keys()[app.nav.lists.programs.at()],
        "run|alice|/usr/bin/nmap"
    );
}

#[test]
fn the_finding_about_a_dropping_spool_walks_to_the_row_that_says_so() {
    let mut app = app();
    app.view = crate::ui::fixture::view_with_launches();
    app.view.found.findings[0].finding_key = "agent.buffer|launches".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Programs);
    assert_eq!(
        app.programs_keys()[app.nav.lists.programs.at()],
        "launches|dropping"
    );
}

#[test]
fn a_jump_to_an_object_that_is_no_longer_in_the_reading_says_so_instead_of_moving() {
    let mut app = app();
    app.view.found.findings[0].finding_key = "port.listen|tcp|0.0.0.0:9999".into();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    assert_eq!(app.nav.at(), Screen::Findings, "nothing moved");
    assert!(drawn(&app).contains("gone since"), "{}", drawn(&app));
}

#[test]
fn escape_after_o_comes_back_to_the_finding_it_was_pressed_on() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    let left_on = app
        .selected_finding()
        .expect("a finding under the cursor")
        .event_id
        .clone();
    app.view.found.findings[2].finding_key = "port.listen|tcp|0.0.0.0:4444".into();

    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.nav.at(), Screen::Ports);

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::Findings);
    assert_eq!(
        app.selected_finding().expect("back on a finding").event_id,
        left_on,
        "back to the row it was pressed on, not to the top of the list"
    );
}

#[test]
fn a_jump_remembers_one_place_and_forgets_it_once_it_has_been_used() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::Findings);

    press(&mut app, KeyCode::Esc);

    assert_eq!(
        app.nav.at(),
        Screen::Home,
        "the second Escape from the same rung is the ladder, not the ring"
    );
}

#[test]
fn walking_into_a_section_from_the_main_screen_leaves_nothing_for_escape_to_come_back_to() {
    let mut app = app();

    press(&mut app, number(Screen::Ports));
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::Home);
}

#[test]
fn changing_section_by_its_number_forgets_where_the_jump_came_from() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);
    press(&mut app, KeyCode::Char('o'));
    assert_eq!(app.nav.at(), Screen::Ports);

    press(&mut app, number(Screen::Accounts));
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::Home);
}

#[test]
fn the_finding_that_is_no_longer_in_the_list_is_named_rather_than_returned_to_silently() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));
    app.view.found.findings.clear();
    app.settle();
    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::Findings, "back where it came from");
    let page = drawn(&app);
    assert!(page.contains("no longer in the list"), "{page}");
    assert!(page.contains("500"), "and how many the agent keeps: {page}");
}

#[test]
fn while_a_jump_is_armed_the_hint_names_the_finding_rather_than_the_main_screen() {
    let mut app = app();
    into(&mut app, Screen::Findings, 80, 30);

    press(&mut app, KeyCode::Char('o'));

    let page = drawn(&app);
    assert!(page.contains("back to the finding"), "{page}");
}
