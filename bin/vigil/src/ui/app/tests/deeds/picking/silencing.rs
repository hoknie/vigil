use ratatui::crossterm::event::KeyCode;
use vigil_model::Severity;

use super::findings::{because, on_the_findings, over_a_configuration, shift};
use crate::ui::Screen;
use crate::ui::app::tests::harness::{a_configuration, drawn_at, into, press, silenced, watching};

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
        page.contains("next round"),
        "the agent takes it up on its next round, and a reader who is not told when will watch the \
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
fn an_object_already_written_down_is_not_written_down_twice_and_the_line_says_so() {
    let (mut app, path) = over_a_configuration();
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");
    press(&mut app, KeyCode::Char('u'));

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours again");
    press(&mut app, KeyCode::Char('u'));
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "and again");

    assert_eq!(
        silenced(&path).len(),
        1,
        "three rounds of the same object, one entry: {:#?}",
        silenced(&path)
    );
}

#[test]
fn an_object_somebody_silenced_already_is_taken_off_the_screen_without_claiming_a_write() {
    let path = a_configuration();
    let mut app = watching(&path);
    into(&mut app, Screen::FINDINGS, 200, 30);
    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");
    assert_eq!(silenced(&path).len(), 1);

    let mut again = crate::ui::fixture::finding("It happened once more", Severity::High);
    again.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
    app.view.found.findings.push(again);
    app.dismissed = crate::ui::Dismissed::default();
    app.settle();

    press(&mut app, KeyCode::Char('d'));
    because(&mut app, "ours");

    let page = drawn_at(&app, 200, 30);
    assert!(page.contains("already silenced in"), "{page}");
    assert_eq!(silenced(&path).len(), 1, "{:#?}", silenced(&path));
    assert!(
        !page.contains("It happened once more"),
        "and it still leaves the screen: {page}"
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
