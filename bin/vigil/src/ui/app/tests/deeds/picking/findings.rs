use ratatui::crossterm::event::{KeyCode, KeyModifiers};

use crate::ui::Screen;
use crate::ui::app::App;
use crate::ui::app::tests::harness::{a_configuration, app, into, press, watching};

pub(super) fn on_the_findings() -> App {
    let mut app = app();
    into(&mut app, Screen::FINDINGS, 200, 30);
    app
}

pub(super) fn over_a_configuration() -> (App, String) {
    let path = a_configuration();
    let mut app = watching(&path);
    into(&mut app, Screen::FINDINGS, 200, 30);
    (app, path)
}

pub(super) fn because(app: &mut App, reason: &str) {
    for character in reason.chars() {
        press(app, KeyCode::Char(character));
    }
    press(app, KeyCode::Enter);
}

pub(super) fn with(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    app.on_key(code, modifiers);
}

pub(super) fn shift(app: &mut App, code: KeyCode) {
    with(app, code, KeyModifiers::SHIFT);
}

pub(super) fn control(app: &mut App, code: KeyCode) {
    with(app, code, KeyModifiers::CONTROL);
}

#[test]
fn a_kind_chosen_from_the_filter_leaves_only_the_findings_of_that_kind_and_says_so() {
    let mut app = on_the_findings();
    app.view.found.findings[1].kind = vigil_model::Kind::from("user.session.new".to_string());
    app.settle();

    press(&mut app, KeyCode::Char('f'));
    let offered = crate::ui::app::tests::harness::drawn_at(&app, 200, 30);
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    let page = crate::ui::app::tests::harness::drawn_at(&app, 200, 30);

    assert!(offered.contains("only port.listen.new (2)"), "{offered}");
    assert!(offered.contains("only user.session.new (1)"), "{offered}");
    assert!(page.contains("A user logged in"), "{page}");
    assert!(!page.contains("A new listening port"), "{page}");
    assert!(page.contains("only user.session.new"), "{page}");
}

#[test]
fn the_findings_sort_by_how_often_each_was_seen() {
    let mut app = on_the_findings();
    app.view.found.findings[2].occurrences = 40;
    app.settle();

    press(&mut app, KeyCode::Char('s'));
    for _ in 0..12 {
        press(&mut app, KeyCode::Down);
    }
    press(&mut app, KeyCode::Enter);
    let page = crate::ui::app::tests::harness::drawn_at(&app, 200, 30);

    assert!(page.contains("sorted by SEEN, largest first"), "{page}");
    let first = page.find("A listening port closed").expect("drawn");
    let second = page.find("A new listening port").expect("drawn");
    assert!(first < second, "{page}");
}
