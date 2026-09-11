use ratatui::crossterm::event::KeyCode;

use crate::ui::{Level, Screen, Subject};

use super::harness::{app, drawn, into, number, press};

#[test]
fn the_console_opens_on_the_main_screen_and_the_arrows_are_in_its_list() {
    let app = app();

    assert_eq!(app.nav.at(), Screen::Home);
    assert_eq!(app.level, Level::List);
    assert!(
        drawn(&app).contains("What this agent watches"),
        "{}",
        drawn(&app)
    );
}

#[test]
fn the_main_screen_opens_the_section_the_cursor_is_on() {
    let mut app = app();
    press(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Enter);

    assert_eq!(app.nav.at(), Screen::Accounts);
    assert_eq!(
        app.level,
        Level::Menu,
        "and lands on that section's top rung, which is its row of lists"
    );
}

#[test]
fn the_right_arrow_opens_a_section_the_way_enter_does() {
    let mut app = app();

    press(&mut app, KeyCode::Right);

    assert_eq!(app.nav.at(), Screen::Ports);
}

#[test]
fn escape_from_the_top_of_a_section_goes_to_the_main_screen_and_nowhere_else() {
    for screen in Screen::ALL {
        let mut app = app();
        press(&mut app, number(*screen));
        assert_eq!(app.nav.at(), *screen);

        press(&mut app, KeyCode::Esc);

        assert_eq!(
            app.nav.at(),
            Screen::Home,
            "Escape from the top of {} went somewhere else",
            screen.name()
        );
        press(&mut app, KeyCode::Esc);
        assert!(
            !app.leaving,
            "and the main screen is the top: people leave with q"
        );
    }
}

#[test]
fn the_main_screen_holds_the_cursor_on_the_section_it_was_left_from() {
    let mut app = app();

    press(&mut app, number(Screen::Startup));
    press(&mut app, KeyCode::Esc);

    let page = drawn(&app);
    assert!(
        page.lines().any(|line| line.contains(" > 4 startup")),
        "coming back put the reader at the top of the list: {page}"
    );
}

#[test]
fn a_number_opens_its_section_from_every_rung_and_lands_on_that_section_s_top_one() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 24);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, number(Screen::Accounts));

    assert_eq!(app.nav.at(), Screen::Accounts);
    assert_eq!(app.level, Level::Menu);

    press(&mut app, number(Screen::Summary));
    assert_eq!(app.nav.at(), Screen::Summary);
    assert_eq!(
        app.level,
        Level::List,
        "a section without a row of lists is entered at its list"
    );
}

#[test]
fn a_digit_leaves_the_submenu_the_way_it_leaves_every_other_level() {
    let mut app = app();
    press(&mut app, number(Screen::Accounts));
    assert_eq!(app.level, Level::Menu);

    press(&mut app, number(Screen::Ports));

    assert_eq!(app.nav.at(), Screen::Ports);
    assert_eq!(app.level, Level::Menu);
}

#[test]
fn a_number_with_no_section_behind_it_changes_nothing() {
    let mut app = app();
    press(&mut app, number(Screen::Ports));

    press(&mut app, KeyCode::Char('9'));

    assert_eq!(app.nav.at(), Screen::Ports);
}

#[test]
fn one_level_in_per_press_and_one_level_out_per_press() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 24);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Enter);
    assert_eq!(app.level, Level::Detail);
    assert!(app.detail_open);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::List);
    assert!(
        app.detail_open,
        "the detail stays beside the list, which is what makes walking the list with it open"
    );

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::Home);
    assert!(!app.detail_open);
}

#[test]
fn the_right_and_left_arrows_do_the_same_as_enter_and_escape_inside_a_section() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 24);

    press(&mut app, KeyCode::Right);
    assert_eq!(app.level, Level::Detail);

    press(&mut app, KeyCode::Left);
    assert_eq!(app.level, Level::List);
    assert!(app.detail_open);

    press(&mut app, KeyCode::Left);
    assert_eq!(app.nav.at(), Screen::Home);
}

#[test]
fn the_arrows_move_whatever_has_them_and_nothing_else() {
    let mut app = app();
    into(&mut app, Screen::Findings, 200, 24);

    press(&mut app, KeyCode::Down);
    assert_eq!(app.nav.findings.at(), 1);
    assert_eq!(app.nav.difference.top(), 0);

    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.nav.findings.at(), 1, "the cursor stayed where it was");
    assert!(app.nav.difference.top() > 0, "and the detail moved instead");
}

#[test]
fn the_summary_has_nothing_behind_its_lines_so_right_is_the_end_of_travel() {
    let mut app = app();
    into(&mut app, Screen::Summary, 200, 24);

    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);

    assert_eq!(app.level, Level::List);
    assert!(!app.detail_open);
}

#[test]
fn a_section_with_a_row_of_lists_puts_it_above_the_rows_and_below_nothing() {
    let mut app = app();
    press(&mut app, number(Screen::Accounts));
    assert_eq!(app.level, Level::Menu, "it is the top rung of the section");
    assert!(
        drawn(&app).contains("[users]"),
        "and the row of them is on the screen: {}",
        drawn(&app)
    );

    press(&mut app, KeyCode::Down);
    assert_eq!(app.level, Level::List);

    press(&mut app, KeyCode::Esc);
    assert_eq!(app.level, Level::Menu, "and back out one rung at a time");
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.nav.at(), Screen::Home);
}

#[test]
fn the_horizontal_arrows_walk_the_row_of_lists_and_leave_the_section_alone() {
    let mut app = app();
    press(&mut app, number(Screen::Accounts));

    press(&mut app, KeyCode::Right);
    assert_eq!(app.nav.lists.accounts.showing(), Subject::Groups);
    assert_eq!(app.nav.at(), Screen::Accounts, "the section did not change");
    assert!(drawn(&app).contains("[groups]"), "{}", drawn(&app));

    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    assert_eq!(
        app.nav.lists.accounts.showing(),
        Subject::LoggedIn,
        "and it wraps"
    );
}
