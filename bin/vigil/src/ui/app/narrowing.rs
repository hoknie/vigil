use super::App;

use crate::ui::types::content::nesting;
use crate::ui::{Cursor, Level, Offset, Screen, Search, Startup};

pub const DETAILS: char = 'd';

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
            Screen::Firewall => Some(&self.firewall_search),
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
            Screen::Firewall => Some(&mut self.firewall_search),
            Screen::Home | Screen::Summary => None,
        }
    }

    pub(super) fn letter(&mut self, key: char) {
        let moved = match self.nav.at() {
            Screen::Home | Screen::Summary if key == DETAILS => {
                self.detail_open = !self.detail_open;
                true
            }
            Screen::Ports => match key {
                'a' => {
                    self.ports_protocols.clear();
                    true
                }
                other => self.ports_protocols.toggle(other),
            },
            Screen::Startup if key == nesting::KEY => self.switch_the_view(),
            _ => false,
        };
        if moved {
            self.nav.difference = Offset::default();
            self.settle();
        }
    }

    fn switch_the_view(&mut self) -> bool {
        if self.nav.lists.startup.showing() != Startup::Units {
            self.message = Some(
                "The tree is a view of the units list: press ← or → to it, then t.".to_string(),
            );
            return false;
        }
        self.startup_nesting.toggle();
        true
    }

    pub(super) fn saying(&self) -> bool {
        self.detail_open || !self.look.interactive()
    }

    pub(super) fn showing_why(&self) -> bool {
        self.nav.at() == Screen::Summary && self.detail_open
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
            Screen::Firewall => self.firewall_search.holding_back(),
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
            Screen::Firewall => {
                self.firewall_search.clear();
                self.nav.firewall = Cursor::default();
            }
            Screen::Home | Screen::Summary => {}
        }
    }
}
