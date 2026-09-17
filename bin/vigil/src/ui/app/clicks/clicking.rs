use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

use crate::ui::app::App;
use crate::ui::{Motion, Target};

impl App {
    pub fn on_mouse(&mut self, event: MouseEvent) {
        if !self.wants_the_mouse() || !self.pointer.ready() {
            return;
        }
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => self.clicked(event.column, event.row),
            MouseEventKind::ScrollDown => self.wheeled(event.column, event.row, Motion::Down),
            MouseEventKind::ScrollUp => self.wheeled(event.column, event.row, Motion::Up),
            _ => {}
        }
    }

    fn clicked(&mut self, column: u16, row: u16) {
        if self.paper.is_some() || self.helping {
            self.paper = None;
            self.helping = false;
            return;
        }
        if self.asking().is_some() {
            return;
        }

        self.message = None;
        let target = self.pointer.first_under(column, row);
        if self.a_popup_is_open() && !target.is_some_and(Target::in_a_popup) {
            self.pressed(KeyCode::Esc);
            return;
        }
        if let Some(target) = target {
            self.press_the_target(target);
        }
    }

    pub(super) fn a_popup_is_open(&self) -> bool {
        self.choosing()
            || self
                .editing
                .as_ref()
                .is_some_and(|editing| editing.dropdown().is_some())
    }

    pub(super) fn pressed(&mut self, code: KeyCode) {
        self.on_key(code, KeyModifiers::NONE);
    }
}
