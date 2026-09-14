use ratatui::crossterm::event::KeyCode;

use crate::ui::Level;
use crate::ui::app::tests::harness::{app, drawn_at, number, press};
use crate::ui::fixture::screen;

pub(super) const SOCKETS: usize = 0;

pub(super) const BY_PROGRAM: usize = 1;

pub(super) fn on_the_sockets(pane: usize) -> crate::ui::App {
    let mut app = app();
    press(&mut app, number(screen("ports")));
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
