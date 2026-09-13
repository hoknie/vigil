use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use vigil_model::Severity;

use crate::ui::{Deed, Level, Screen};

use super::harness::{a_configuration, app, drawn, drawn_at, into, press, silenced, watching};
use crate::ui::app::App;

fn on_the_findings() -> App {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 200, 30);
    app
}

fn over_a_configuration() -> (App, String) {
    let path = a_configuration();
    let mut app = watching(&path);
    into(&mut app, Screen::FINDINGS, 200, 30);
    (app, path)
}

fn because(app: &mut App, reason: &str) {
    for character in reason.chars() {
        press(app, KeyCode::Char(character));
    }
    press(app, KeyCode::Enter);
}

fn with(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.on_key(code, modifiers);
}

fn shift(app: &mut App, code: KeyCode) {
    with(app, code, KeyModifiers::SHIFT);
}

fn control(app: &mut App, code: KeyCode) {
    with(app, code, KeyModifiers::CONTROL);
}

#[test]
fn a_console_nobody_has_picked_anything_on_draws_no_bar_and_no_gutter_for_one() {
    let app = on_the_findings();

    let page = drawn(&app);

    assert!(!page.contains("picked"), "{page}");
    assert!(
        page.contains("A new listening port"),
        "the table is drawn as it always was: {page}"
    );
}

#[test]
fn shift_with_an_arrow_picks_the_rows_it_walks_over_and_says_how_many() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Down);

    let page = drawn(&app);
    assert!(page.contains("2 picked"), "{page}");
    assert!(
        page.contains(Deed::Remove.named()),
        "a bar with no deed on it is a bar that does nothing: {page}"
    );
}

#[test]
fn a_picked_row_is_marked_with_a_character_and_not_only_a_colour() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Down);

    let page = drawn(&app);
    let marked = page.lines().filter(|line| line.contains('\u{d7}')).count();
    assert_eq!(
        marked, 2,
        "a reader with no colour has to see which two rows a deed would reach: {page}"
    );
}

#[test]
fn control_with_an_arrow_gathers_rows_that_are_not_beside_each_other() {
    let mut app = on_the_findings();

    control(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    control(&mut app, KeyCode::Down);

    let page = drawn(&app);
    assert!(page.contains("2 picked"), "{page}");
    let lines: Vec<&str> = page
        .lines()
        .filter(|line| line.contains("09:00:00"))
        .collect();
    assert!(
        lines[0].contains('\u{d7}') && !lines[1].contains('\u{d7}') && lines[2].contains('\u{d7}'),
        "the row between the two picked ones was picked as well: {page}"
    );
}

#[test]
fn the_key_under_the_cursor_picks_one_row_where_a_terminal_sends_no_modifiers() {
    let mut app = on_the_findings();

    press(&mut app, KeyCode::Char('x'));

    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('x'));

    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}

#[test]
fn a_whole_screenful_goes_on_and_comes_off_with_the_same_key() {
    let mut app = on_the_findings();

    press(&mut app, KeyCode::Char('a'));
    assert!(drawn(&app).contains("3 picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Char('a'));
    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}

#[test]
fn removing_a_finding_writes_it_into_the_configuration_and_takes_it_off_the_screen() {
    let (mut app, path) = over_a_configuration();
    shift(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "the staging api, expected here");

    let page = drawn_at(&app, 200, 30);
    assert!(!page.contains("A new listening port"), "{page}");
    assert!(!page.contains("A user logged in"), "{page}");
    let held = silenced(&path);
    assert_eq!(held.len(), 1, "{held:#?}");
    assert!(
        held[0].contains("port.listen|tcp|0.0.0.0:4444"),
        "{held:#?}"
    );
    assert!(
        held[0].contains("the staging api, expected here"),
        "the reason is what the file carries, and it is the reader's own words: {held:#?}"
    );
    assert!(
        page.contains("systemctl try-restart"),
        "the daemon reads that file at start, and a reader who is not told will watch the \
         same finding come back: {page}"
    );
}

#[test]
fn silencing_one_row_takes_every_row_about_the_same_object_off_the_screen() {
    let (mut app, path) = over_a_configuration();
    let before = drawn_at(&app, 200, 30);
    assert!(before.contains("A new listening port"), "{before}");
    assert!(before.contains("A user logged in"), "{before}");
    assert!(before.contains("A listening port closed"), "{before}");

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let page = drawn_at(&app, 200, 30);
    assert!(
        !page.contains("A user logged in") && !page.contains("A listening port closed"),
        "the entry names the object, and the fixture raises three findings about one object: \
         a row left on the screen is a row the agent has stopped reporting about: {page}"
    );
    assert!(page.contains("0 shown of 3 held"), "{page}");
    assert!(
        page.contains("3 silenced from here"),
        "and the count under the table is rows, so shown plus silenced is what the agent \
         holds: {page}"
    );
    assert_eq!(silenced(&path).len(), 1);
}

#[test]
fn a_finding_raised_about_a_silenced_object_before_the_daemon_restarts_is_not_drawn_either() {
    let (mut app, _path) = over_a_configuration();
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let mut again = crate::ui::fixture::finding("It happened once more", Severity::High);
    again.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
    app.view.found.findings.push(again);
    app.settle();

    let page = drawn_at(&app, 200, 30);
    assert!(
        !page.contains("It happened once more"),
        "the agent goes on raising it until it is restarted, and a reader who silenced the \
         object has said they do not want to see it: {page}"
    );
}

#[test]
fn the_object_is_written_down_once_however_many_rows_of_it_were_picked() {
    let (mut app, path) = over_a_configuration();
    press(&mut app, KeyCode::Char('a'));

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(
        silenced(&path).len(),
        1,
        "the fixture raises three findings about one object, and one object is one entry"
    );
}

#[test]
fn nothing_is_written_until_a_reason_is_given_and_escape_writes_nothing_at_all() {
    let (mut app, path) = over_a_configuration();

    press(&mut app, KeyCode::Char('d'));
    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("why is"), "{page}");
    assert!(page.contains("Esc leaves it"), "{page}");
    assert!(
        page.contains("A new listening port"),
        "nothing has left the screen yet: {page}"
    );

    press(&mut app, KeyCode::Esc);

    assert!(silenced(&path).is_empty(), "{:#?}", silenced(&path));
    assert!(drawn_at(&app, 200, 30).contains("A new listening port"));
}

#[test]
fn a_reason_nobody_typed_is_refused_and_the_question_stays_open() {
    let (mut app, path) = over_a_configuration();
    press(&mut app, KeyCode::Char('d'));

    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("needs a reason"), "{page}");
    assert!(silenced(&path).is_empty());
    press(&mut app, KeyCode::Char('x'));
    press(&mut app, KeyCode::Enter);
    assert_eq!(silenced(&path).len(), 1, "and it can still be answered");
}

#[test]
fn a_configuration_this_console_cannot_write_takes_nothing_off_the_screen_and_says_why() {
    let mut app = on_the_findings();

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("Nothing was silenced"), "{page}");
    assert!(page.contains("/etc/vigil/vigil.yaml"), "{page}");
    assert!(
        page.contains("A new listening port"),
        "a row that is still reported must not look as though it is not: {page}"
    );
}

#[test]
fn bringing_it_back_takes_the_entry_out_of_the_configuration_as_well_as_back_on_the_screen() {
    let (mut app, path) = over_a_configuration();
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");
    assert_eq!(silenced(&path).len(), 1);

    press(&mut app, KeyCode::Char('u'));

    assert!(silenced(&path).is_empty(), "{:#?}", silenced(&path));
    assert!(drawn_at(&app, 200, 30).contains("A new listening port"));
    assert_eq!(
        std::fs::read_to_string(&path).expect("readable"),
        super::harness::SHIPPED,
        "what the console wrote, the console takes out, and the file is the one it started as"
    );
}

#[test]
fn the_panel_of_a_finding_offers_the_deed_and_the_key_that_does_it() {
    let (mut app, path) = over_a_configuration();

    press(&mut app, KeyCode::Enter);

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("ACTIONS"), "{page}");
    assert!(page.contains(Deed::Remove.alone()), "{page}");

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(
        silenced(&path).len(),
        1,
        "the key the panel offers does nothing from it"
    );
}

#[test]
fn the_file_a_finding_is_silenced_in_is_the_one_the_daemon_says_it_read() {
    let path = a_configuration();
    let mut app = app();
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = Some(path.clone());
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(
        silenced(&path).len(),
        1,
        "the daemon is started with the file named on its command line, and a console that \
         wrote the packaged default instead would silence nothing and say it had"
    );
}

#[test]
fn a_file_named_on_the_command_line_outranks_the_one_the_daemon_answered_with() {
    let named = a_configuration();
    let path = a_configuration();
    let mut app = watching(&named);
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = Some(path.clone());
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    assert_eq!(silenced(&named).len(), 1);
    assert!(silenced(&path).is_empty());
}

#[test]
fn an_agent_that_does_not_say_which_file_it_read_is_answered_with_the_one_the_package_installs() {
    let mut app = app();
    if let Some(status) = app.view.status.as_mut() {
        status.agent.configuration_path = None;
    }
    into(&mut app, Screen::FINDINGS, 200, 30);

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("/etc/vigil/vigil.yaml"), "{page}");
}

#[test]
fn the_summary_says_which_file_a_finding_would_be_silenced_in() {
    let mut app = app();
    press(&mut app, KeyCode::Char('8'));

    let page = drawn_at(&app, 200, 40);
    assert!(page.contains("configuration"), "{page}");
    assert!(page.contains("/etc/vigil/vigil.yaml"), "{page}");
    assert!(
        page.contains("silenced here"),
        "the file that key writes to is worth naming before the key is pressed: {page}"
    );
}

#[test]
fn escape_lets_go_of_what_is_picked_before_it_leaves_the_screen() {
    let mut app = on_the_findings();
    shift(&mut app, KeyCode::Down);

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::FINDINGS, "it left the screen as well");
    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));

    press(&mut app, KeyCode::Esc);

    assert_eq!(app.nav.at(), Screen::HOME);
}

#[test]
fn walking_off_the_findings_lets_go_of_the_rows_that_cannot_be_seen_any_more() {
    let mut app = on_the_findings();
    press(&mut app, KeyCode::Char('a'));

    press(&mut app, KeyCode::Char('1'));
    press(&mut app, KeyCode::Char('9'));

    assert!(
        !drawn(&app).contains("picked"),
        "a deed must not reach rows a reader picked on a screen they have left: {}",
        drawn(&app)
    );
}

#[test]
fn a_search_that_hides_a_picked_row_lets_go_of_it_rather_than_acting_on_it_unseen() {
    let mut app = on_the_findings();
    press(&mut app, KeyCode::Char('a'));

    press(&mut app, KeyCode::Char('/'));
    for character in "logged in".chars() {
        press(&mut app, KeyCode::Char(character));
    }
    press(&mut app, KeyCode::Enter);

    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));
}

#[test]
fn shift_at_the_top_of_the_list_picks_upwards_instead_of_leaving_the_screen() {
    let mut app = on_the_findings();

    shift(&mut app, KeyCode::Up);

    assert_eq!(app.nav.at(), Screen::FINDINGS);
    assert_eq!(app.level, Level::List);
    assert!(drawn(&app).contains("1 picked"), "{}", drawn(&app));
}

#[test]
fn the_bar_and_the_table_stay_inside_the_terminal_they_are_drawn_on() {
    for (width, height) in [(80u16, 24u16), (200, 30), (60, 12)] {
        let mut app = app();
        into(&mut app, Screen::FINDINGS, width, height);
        drawn_at(&app, width, height);
        shift(&mut app, KeyCode::Down);

        let page = drawn_at(&app, width, height);
        for line in page.lines() {
            assert!(
                line.chars().count() <= width as usize,
                "{width}x{height}: {line}"
            );
        }
        assert!(page.contains("picked"), "{width}x{height}: {page}");
    }
}

#[test]
fn nothing_is_picked_on_a_screen_that_is_not_the_findings() {
    let mut app = app();
    into(
        &mut app,
        Screen::parse("ports").expect("a section"),
        200,
        30,
    );

    shift(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Char('x'));

    assert!(!drawn(&app).contains("picked"), "{}", drawn(&app));
}
