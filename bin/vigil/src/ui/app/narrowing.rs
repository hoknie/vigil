use super::App;

use crate::ui::types::content::nesting;
use crate::ui::{Cursor, Level, Offset, Panes, Screen, Search, Startup, holding};

pub const DETAILS: char = 'd';

impl App {
    pub(super) fn typing(&self) -> bool {
        self.search().is_some_and(Search::typing)
    }

    pub(super) fn search(&self) -> Option<&Search> {
        match self.nav.at() {
            Screen::Findings => Some(self.filter.search()),
            screen if holding(screen.name()).is_some() => self.panes().map(Panes::search),
            Screen::Programs => Some(self.nav.lists.programs.search()),
            Screen::Startup => Some(self.nav.lists.startup.search()),
            Screen::System => Some(self.nav.lists.system.search()),
            _ => None,
        }
    }

    pub(super) fn search_mut(&mut self) -> Option<&mut Search> {
        match self.nav.at() {
            Screen::Findings => Some(self.filter.search_mut()),
            screen if holding(screen.name()).is_some() => self.panes_mut().map(Panes::search_mut),
            Screen::Programs => Some(self.nav.lists.programs.search_mut()),
            Screen::Startup => Some(self.nav.lists.startup.search_mut()),
            Screen::System => Some(self.nav.lists.system.search_mut()),
            _ => None,
        }
    }

    pub(super) fn letter(&mut self, key: char) {
        let moved = match self.nav.at() {
            Screen::Home | Screen::Summary if key == DETAILS => {
                self.detail_open = !self.detail_open;
                true
            }
            screen if holding(screen.name()).is_some() => self.switch_a_kind(key),
            Screen::Startup if key == nesting::KEY => self.switch_the_view(),
            _ => false,
        };
        if moved {
            self.nav.difference = Offset::default();
            self.settle();
        }
    }

    fn switch_a_kind(&mut self, key: char) -> bool {
        if key == 'a' {
            if let Some(panes) = self.panes_mut() {
                panes.show_every_kind();
            }
            return true;
        }
        let Some(pane) = self.pane() else {
            return false;
        };
        let Some(toggle) = pane.toggles().into_iter().find(|toggle| toggle.key == key) else {
            return false;
        };
        if let Some(panes) = self.panes_mut() {
            panes.toggle(toggle.name);
        }
        true
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
            screen if holding(screen.name()).is_some() => self.panes().is_some_and(|panes| {
                panes.search().holding_back() || (everything && panes.hiding())
            }),
            Screen::Programs => self.nav.lists.programs.search().holding_back(),
            Screen::Startup => self.nav.lists.startup.search().holding_back(),
            Screen::System => self.nav.lists.system.search().holding_back(),
            _ => false,
        }
    }

    pub(super) fn widen(&mut self) {
        match self.nav.at() {
            Screen::Findings => {
                self.filter.clear();
                self.nav.findings = Cursor::default();
            }
            screen if holding(screen.name()).is_some() => {
                let menu = self.level == Level::Menu;
                if let Some(panes) = self.panes_mut() {
                    panes.widen();
                    if menu {
                        panes.show_every_kind();
                    }
                }
            }
            Screen::Programs => self.nav.lists.programs.widen(),
            Screen::Startup => self.nav.lists.startup.widen(),
            Screen::System => self.nav.lists.system.widen(),
            _ => {}
        }
    }
}
