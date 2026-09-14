use super::App;

use crate::ui::{Cursor, Level, Offset, Panes, Screen, Search, holding};

use super::deeds::{DELETE, EDIT, MARK, NEW, NOTHING_TO_CHANGE, SUPPRESS, UNMARK_EVERY};

pub const DETAILS: char = 'd';

impl App {
    pub(super) fn typing(&self) -> bool {
        self.asking().is_some() || self.search().is_some_and(Search::typing)
    }

    pub(super) fn search(&self) -> Option<&Search> {
        match self.nav.at() {
            Screen::FINDINGS => Some(self.filter.search()),
            screen if screen.draws_a_reading() => self.panes().map(Panes::search),
            _ => None,
        }
    }

    pub(super) fn search_mut(&mut self) -> Option<&mut Search> {
        match self.nav.at() {
            Screen::FINDINGS => Some(self.filter.search_mut()),
            screen if screen.draws_a_reading() => self.panes_mut().map(Panes::search_mut),
            _ => None,
        }
    }

    pub(super) fn letter(&mut self, key: char) {
        let moved = match self.nav.at() {
            Screen::HOME | Screen::SUMMARY if key == DETAILS => {
                self.detail_open = !self.detail_open;
                true
            }
            Screen::FINDINGS => self.picking_key(key),
            screen if holding(screen.name()).is_some() => match key {
                MARK => self.mark_under_the_cursor(),
                UNMARK_EVERY => self.unmark_everything(),
                SUPPRESS => self.show_the_suppressions(),
                NEW | EDIT | DELETE if self.changes_offered().is_some() => {
                    self.changing_by_key(key)
                }
                _ => {
                    let switched = self.switch_a_kind(key);
                    if !switched && self.message.is_none() && [NEW, EDIT, DELETE].contains(&key) {
                        self.message = Some(NOTHING_TO_CHANGE.to_string());
                    }
                    switched
                }
            },
            _ => false,
        };
        if moved {
            self.nav.difference = Offset::default();
            self.settle();
        }
    }

    fn switch_a_kind(&mut self, key: char) -> bool {
        if self.switch_the_view(key) {
            return true;
        }
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

    fn switch_the_view(&mut self, key: char) -> bool {
        let Some(pane) = self.pane() else {
            return false;
        };
        let every = pane.arrangements();
        if !every.iter().any(|one| one.key == key) {
            return self.said_where_the_view_lives(key);
        }

        let here = self
            .panes()
            .and_then(Panes::arranged)
            .and_then(|name| every.iter().position(|one| one.name == name))
            .unwrap_or(0);
        let next = every[(here + 1) % every.len()].name;
        if let Some(panes) = self.panes_mut() {
            panes.arrange(next);
        }
        true
    }

    fn said_where_the_view_lives(&mut self, key: char) -> bool {
        let Some(section) = self.section() else {
            return false;
        };
        let Some(elsewhere) = section
            .panes()
            .iter()
            .find(|pane| pane.arrangements().iter().any(|one| one.key == key))
            .map(|pane| pane.name().to_string())
        else {
            return false;
        };

        self.message = Some(format!(
            "That view belongs to the {elsewhere} list: press \u{2190} or \u{2192} to it, then {key}."
        ));
        false
    }

    pub(super) fn saying(&self) -> bool {
        self.detail_open || !self.look.interactive()
    }

    pub(super) fn showing_why(&self) -> bool {
        self.nav.at() == Screen::SUMMARY && self.detail_open
    }

    pub(super) fn narrowed(&self) -> bool {
        let everything = self.level == Level::Menu;
        match self.nav.at() {
            Screen::FINDINGS => self.filter.holding_back(),
            screen if screen.draws_a_reading() => self.panes().is_some_and(|panes| {
                panes.search().holding_back() || (everything && panes.hiding())
            }),
            _ => false,
        }
    }

    pub(super) fn widen(&mut self) {
        match self.nav.at() {
            Screen::FINDINGS => {
                self.filter.clear();
                self.nav.findings = Cursor::default();
            }
            screen if screen.draws_a_reading() => {
                let menu = self.level == Level::Menu;
                if let Some(panes) = self.panes_mut() {
                    panes.widen();
                    if menu {
                        panes.show_every_kind();
                    }
                }
            }
            _ => {}
        }
    }
}
