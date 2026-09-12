use super::App;

use crate::ui::helpers::layout::split;
use crate::ui::screens::{findings, home, ports};
use crate::ui::{Level, Offset, Origin, Program, Rungs, Screen, Startup, Subject, System};

impl App {
    pub(super) fn rungs(&self) -> Rungs {
        Rungs::new(
            matches!(
                self.nav.at(),
                Screen::Accounts
                    | Screen::Ports
                    | Screen::Programs
                    | Screen::Startup
                    | Screen::System
            ),
            self.has_detail(),
        )
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
        if self.level != Level::Detail && self.narrowed() {
            self.widen();
            return;
        }
        if self.level == Level::Detail {
            self.level = Level::List;
            if split::beside(self.body.get()).is_none() {
                self.detail_open = false;
            }
            return;
        }
        if self.detail_open {
            self.detail_open = false;
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
        if self.nav.at() == Screen::Home {
            return;
        }
        match self.nav.came_back() {
            Some(origin) => {
                self.arrive();
                self.point_back_at(&origin);
            }
            None => {
                self.remember_section(self.nav.at());
                self.nav.visit(Screen::Home);
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
        if self.nav.at() == Screen::Home {
            self.open_section();
            return;
        }
        let deeper = self.level.deeper(self.rungs());
        if deeper == Level::Detail && self.level != Level::Detail {
            self.nav.difference = Offset::default();
            self.detail_open = true;
        }
        self.level = deeper;
    }

    fn open_section(&mut self) {
        let Some(row) = self.section_under_cursor() else {
            return;
        };
        match row.opens {
            Some(screen) => self.visit(screen),
            None => self.message = Some(home::nothing_to_open(&row.name)),
        }
    }

    pub(super) fn sideways(&mut self, by: isize) {
        if !self.level.is_a_row_of_names() {
            return match by > 0 {
                true => self.open(),
                false => self.go_back(),
            };
        }
        match self.nav.at() {
            Screen::Ports => self.nav.lists.ports.step_along(by, ports::Arrangement::ALL),
            Screen::Programs => self.nav.lists.programs.step_along(by, Program::ALL),
            Screen::System => self.nav.lists.system.step_along(by, System::ALL),
            Screen::Startup => {
                let shown = Startup::on(&self.view);
                self.nav.lists.startup.step_along(by, &shown);
            }
            _ => {
                let shown = Subject::on(&self.view);
                self.nav.lists.accounts.step_along(by, &shown);
            }
        }
        self.detail_open = false;
        self.nav.difference = Offset::default();
        self.refresh_wanted = true;
    }
}
