use ratatui::crossterm::event::KeyCode;

use crate::ui::app::App;
use crate::ui::{Level, Motion, Screen, Target};

impl App {
    pub(super) fn wheeled(&mut self, column: u16, row: u16, motion: Motion) {
        if self.a_popup_is_open() || self.editing.is_some() || self.asking().is_some() {
            return;
        }
        if self.graph.is_some() || self.history.is_some() {
            self.pressed(match motion {
                Motion::Up => KeyCode::Up,
                _ => KeyCode::Down,
            });
            return;
        }

        let under = self
            .pointer
            .under(column, row)
            .into_iter()
            .find(|target| matches!(target, Target::List | Target::Detail));
        match under {
            Some(Target::Detail) => self.scroll_the_detail(motion),
            Some(Target::List) => self.scroll_the_list(motion),
            _ => {}
        }
    }

    fn scroll_the_detail(&mut self, motion: Motion) {
        if self.nav.at() == Screen::HOME {
            return;
        }
        let Some(area) = self.detail_area() else {
            return;
        };
        let total = self.detail_height(area);
        self.nav
            .difference
            .step(motion, total, area.height as usize);
    }

    fn scroll_the_list(&mut self, motion: Motion) {
        self.level = Level::List;
        if !self.climbing(motion) {
            self.move_within(motion);
        }
        self.settle();
    }
}
