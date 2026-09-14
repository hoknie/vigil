use super::App;

use crate::ui::helpers::layout::split;
use crate::ui::screens::{findings, home};
use crate::ui::{Level, Offset, Origin, Rungs, Screen};

impl App {
    pub(super) fn rungs(&self) -> Rungs {
        let menu = self.section().is_some() && self.shown_panes().len() > 1;

        Rungs::new(menu, self.has_detail())
    }

    pub(super) fn visit(&mut self, screen: Screen) {
        if self.nav.at() == screen {
            return;
        }
        self.remember_section(screen);
        self.nav.visit(screen);
        self.arrive();
        self.refresh_wanted = true;
    }

    pub(super) fn arrive(&mut self) {
        self.picked.clear();
        self.level = Level::top(self.rungs());
        self.detail_open = false;
        self.gone = None;
    }

    pub(super) fn remember_section(&mut self, screen: Screen) {
        let keys = home::keys(&self.view);
        self.nav.sections.point_at(screen.name(), &keys);
    }

    pub(super) fn section_under_cursor(&self) -> Option<home::Row> {
        home::rows(&self.view)
            .into_iter()
            .nth(self.nav.sections.at())
    }

    pub(super) fn go_back(&mut self) {
        self.go_back_from(true);
    }

    pub(super) fn leave_the_row(&mut self) {
        self.go_back_from(false);
    }

    fn go_back_from(&mut self, folding: bool) {
        if self.level != Level::Detail && self.narrowed() {
            self.widen();
            return;
        }
        if self.level == Level::Detail {
            self.level = Level::List;
            self.rest_the_buttons();
            if split::beside(self.body.get()).is_none() {
                self.detail_open = false;
            }
            return;
        }
        if self.detail_open {
            self.detail_open = false;
            return;
        }
        if folding && self.level == Level::List && self.close_the_branch() {
            return;
        }
        if self.level == Level::List && self.nav.came_from().is_some() {
            self.leave_section();
            return;
        }
        match self.level.shallower(self.rungs()) {
            Some(level) => self.level = level,
            None => self.leave_section(),
        }
    }

    fn leave_section(&mut self) {
        if self.nav.at() == Screen::HOME {
            return;
        }
        match self.nav.came_back() {
            Some(origin) => {
                self.arrive();
                self.point_back_at(&origin);
            }
            None => {
                self.remember_section(self.nav.at());
                self.nav.visit(Screen::HOME);
                self.arrive();
                self.refresh_wanted = true;
            }
        }
    }

    fn point_back_at(&mut self, origin: &Origin) {
        self.level = Level::List;
        self.refresh_wanted = true;
        let passing = self.passing();
        let keys = findings::keys(&passing);
        if !self.nav.findings.point_at(origin.key(), &keys) {
            self.message = Some(format!(
                "That finding is no longer in the list: the agent keeps the last {} it raised.",
                self.view.found.capacity
            ));
        }
    }

    pub(super) fn open(&mut self) {
        if self.nav.at() == Screen::HOME {
            self.open_section();
            return;
        }
        if self.press_the_button() {
            return;
        }
        if self.level == Level::List && self.open_the_branch() {
            return;
        }
        if self.level == Level::List && self.show_the_panel() {
            return;
        }
        let deeper = self.level.deeper(self.rungs());
        if deeper == Level::Detail && self.level != Level::Detail {
            self.nav.difference = Offset::default();
            self.detail_open = true;
            self.rest_the_buttons();
        }
        self.level = deeper;
    }

    fn show_the_panel(&mut self) -> bool {
        if self.detail_open || !self.rungs().detail {
            return false;
        }
        if split::beside(self.body.get()).is_none() {
            return false;
        }

        self.nav.difference = Offset::default();
        self.detail_open = true;
        true
    }

    fn open_section(&mut self) {
        let Some(row) = self.section_under_cursor() else {
            return;
        };
        let screen = row.opens;

        self.visit(screen);
        if let Some(named) = row.opens_reading {
            self.onto_the_reading(screen, &named);
        }
    }

    fn onto_the_reading(&mut self, screen: Screen, named: &str) {
        let Some(section) = self.section_of(screen) else {
            return;
        };
        let Some(at) = section
            .panes()
            .iter()
            .position(|pane| pane.reads() == named)
        else {
            return;
        };
        if let Some(panes) = self.panes_of_mut(screen) {
            panes.show(at);
        }
    }

    fn open_the_branch(&mut self) -> bool {
        let Some(row) = self.pane_row_under_the_cursor() else {
            return false;
        };
        if !row.opens() || row.opened {
            return false;
        }
        if let Some(panes) = self.panes_mut() {
            panes.open(&row.key, true);
        }
        self.settle();
        true
    }

    pub(super) fn close_the_branch(&mut self) -> bool {
        let Some(row) = self.pane_row_under_the_cursor() else {
            return false;
        };
        if row.opens() && row.opened {
            if let Some(panes) = self.panes_mut() {
                panes.open(&row.key, false);
            }
            self.settle();
            return true;
        }

        let Some(heading) = row.gathered_under.clone() else {
            return false;
        };
        if let Some(panes) = self.panes_mut() {
            panes.open(&heading, false);
        }
        self.point_the_cursor_at(&heading);
        true
    }

    fn point_the_cursor_at(&mut self, key: &str) {
        let rows = self.pane_keys();
        if let Some(panes) = self.panes_mut() {
            panes.cursor_mut().point_at(key, &rows);
        }
        self.settle();
    }

    pub(super) fn sideways(&mut self, by: isize) {
        if self.level == Level::Detail {
            let moved = match by > 0 {
                true => self.deeper_into_the_buttons(),
                false => self.back_out_of_the_buttons(),
            };
            if moved {
                return;
            }
        }
        if !self.level.is_a_row_of_names() {
            return match by > 0 {
                true => self.open(),
                false => self.go_back(),
            };
        }
        match self.nav.at() {
            screen if screen.draws_a_reading() => {
                let shown = self.shown_panes();
                if let Some(panes) = self.panes_mut() {
                    panes.step_along(by, &shown);
                }
            }
            _ => {}
        }
        self.detail_open = false;
        self.nav.difference = Offset::default();
        self.refresh_wanted = true;
    }
}
