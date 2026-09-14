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
