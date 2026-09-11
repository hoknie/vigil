use super::App;

use crate::ui::{Cursor, Level, Offset, Screen, Search};

impl App {
    pub(super) fn typing(&self) -> bool {
        self.search().is_some_and(Search::typing)
    }

    pub(super) fn search(&self) -> Option<&Search> {
        match self.nav.at() {
            Screen::Findings => Some(self.filter.search()),
            Screen::Ports => Some(self.nav.lists.ports.search()),
            Screen::Accounts => Some(self.nav.lists.accounts.search()),
            Screen::Programs => Some(self.nav.lists.programs.search()),
            Screen::Startup => Some(self.nav.lists.startup.search()),
            Screen::Home | Screen::Summary => None,
        }
    }

    pub(super) fn search_mut(&mut self) -> Option<&mut Search> {
        match self.nav.at() {
            Screen::Findings => Some(self.filter.search_mut()),
            Screen::Ports => Some(self.nav.lists.ports.search_mut()),
            Screen::Accounts => Some(self.nav.lists.accounts.search_mut()),
            Screen::Programs => Some(self.nav.lists.programs.search_mut()),
            Screen::Startup => Some(self.nav.lists.startup.search_mut()),
            Screen::Home | Screen::Summary => None,
        }
    }

    pub(super) fn letter(&mut self, key: char) {
        if self.nav.at() != Screen::Ports {
            return;
        }
        let moved = match key {
            'a' => {
                self.ports_protocols.clear();
                true
            }
            other => self.ports_protocols.toggle(other),
        };
        if moved {
            self.nav.difference = Offset::default();
            self.settle();
        }
    }

    pub(super) fn narrowed(&self) -> bool {
        let everything = self.level == Level::Menu;
        match self.nav.at() {
            Screen::Findings => self.filter.holding_back(),
            Screen::Ports => {
                self.nav.lists.ports.search().holding_back()
                    || (everything && self.ports_protocols.holding_back())
            }
            Screen::Accounts => self.nav.lists.accounts.search().holding_back(),
            Screen::Programs => self.nav.lists.programs.search().holding_back(),
            Screen::Startup => self.nav.lists.startup.search().holding_back(),
            Screen::Home | Screen::Summary => false,
        }
    }

    pub(super) fn widen(&mut self) {
        match self.nav.at() {
            Screen::Findings => {
                self.filter.clear();
                self.nav.findings = Cursor::default();
            }
            Screen::Ports => {
                self.nav.lists.ports.widen();
                if self.level == Level::Menu {
                    self.ports_protocols.clear();
                }
            }
            Screen::Accounts => self.nav.lists.accounts.widen(),
            Screen::Programs => self.nav.lists.programs.widen(),
            Screen::Startup => self.nav.lists.startup.widen(),
            Screen::Home | Screen::Summary => {}
        }
    }
}
