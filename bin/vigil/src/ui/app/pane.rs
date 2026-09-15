use std::rc::Rc;

use vigil_view::{Pane, Piece, RowKey, Section};

use super::App;

use crate::ui::screens::pane;
use crate::ui::screens::unknown::Unknown;
use crate::ui::{Arrows, Panes, Reading, Screen, holding};

impl App {
    pub(super) fn section(&self) -> Option<Box<dyn Section>> {
        self.section_of(self.nav.at())
    }

    pub(super) fn section_of(&self, screen: Screen) -> Option<Box<dyn Section>> {
        match screen == Screen::UNKNOWN {
            true => Some(Box::new(Unknown::of(&self.view))),
            false => holding(screen.name()),
        }
    }

    pub(super) fn panes(&self) -> Option<&Panes> {
        self.nav.lists.of(self.nav.at().name())
    }

    pub(super) fn panes_mut(&mut self) -> Option<&mut Panes> {
        let screen = self.nav.at().name();
        self.nav.lists.of_mut(screen)
    }

    pub(super) fn panes_of_mut(&mut self, screen: Screen) -> Option<&mut Panes> {
        self.nav.lists.of_mut(screen.name())
    }

    pub(super) fn pane_keys_of(&self, screen: Screen) -> Vec<String> {
        let (Some(section), Some(panes)) =
            (self.section_of(screen), self.nav.lists.of(screen.name()))
        else {
            return Vec::new();
        };
        let every = section.panes();
        let Some(pane) = every.get(panes.showing().min(every.len().saturating_sub(1))) else {
            return Vec::new();
        };
        let marked = panes.marked();
        let branches = panes.opened();
        let showing = pane::Showing {
            listed: None,
            tally: None,
            at: panes.showing(),
            search: panes.search(),
            hidden: panes.hidden(),
            cursor: panes.at(),
            arrows: Arrows::at(self.level),
            sorting: self.sorted(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
            marked,
            opened: branches,
            only: panes.only(),
        };
        let hidden = showing.hidden();
        let opened = showing.opened();

        pane::rows(
            &self.view,
            pane.as_ref(),
            &pane::asked(&showing, &hidden, &opened),
        )
        .into_iter()
        .map(|row| row.key)
        .collect()
    }

    pub(super) fn row_named(&self, screen: Screen, named: &str) -> Option<String> {
        let section = self.section_of(screen)?;

        section.panes().iter().find_map(|pane| {
            let Reading::Taken(reading) = self.view.reading(pane.reads()) else {
                return None;
            };
            pane.row_for(reading, named)
        })
    }

    pub(super) fn pane_of_holding(&self, screen: Screen, key: &str) -> Option<usize> {
        let section = self.section_of(screen)?;

        section.panes().iter().position(|pane| {
            let Reading::Taken(reading) = self.view.reading(pane.reads()) else {
                return false;
            };
            pane.shown(reading) && pane.holds(reading, key)
        })
    }

    pub(super) fn shown_panes(&self) -> Vec<usize> {
        let Some(section) = self.section() else {
            return Vec::new();
        };

        section
            .panes()
            .iter()
            .enumerate()
            .filter(|(_, pane)| match self.view.reading(pane.reads()) {
                Reading::Taken(reading) => pane.shown(reading),
                _ => true,
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(super) fn pane(&self) -> Option<Box<dyn Pane>> {
        let section = self.section()?;
        let panes = self.panes()?;
        let mut every = section.panes();
        let at = panes.showing().min(every.len().saturating_sub(1));
        match every.is_empty() {
            true => None,
            false => Some(every.remove(at)),
        }
    }

    pub(super) fn showing_pane(&self) -> Option<pane::Showing<'_>> {
        let panes = self.panes()?;
        let pane = self.pane()?;

        Some(pane::Showing {
            listed: None,
            tally: None,
            at: panes.showing(),
            search: panes.search(),
            hidden: panes.hidden(),
            cursor: panes.at(),
            arrows: Arrows::at(self.level),
            sorting: self.sorted(),
            note: self.view.collector_note(pane.reads()),
            elsewhere: panes.narrowed_elsewhere(),
            gone: self.gone.as_ref(),
            arranged: arranged(pane.as_ref(), panes.arranged()),
            marked: panes.marked(),
            opened: panes.opened(),
            only: panes.only(),
        })
    }

    pub(super) fn printing_pane<'a>(
        &'a self,
        at: usize,
        search: &'a crate::ui::Search,
    ) -> Option<pane::Showing<'a>> {
        let panes = self.panes()?;

        Some(pane::Showing {
            listed: None,
            tally: None,
            at,
            search,
            hidden: panes.hidden(),
            cursor: 0,
            arrows: Arrows::Away,
            sorting: self.sorted(),
            note: None,
            elsewhere: 0,
            gone: None,
            arranged: None,
            marked: Vec::new(),
            opened: panes.opened(),
            only: panes.only(),
        })
    }

    pub(super) fn pane_row_under_the_cursor(&self) -> Option<RowKey> {
        let at = self.panes()?.at();
        self.pane_rows().get(at).cloned()
    }

    #[cfg(test)]
    pub(super) fn pane_keys(&self) -> Vec<String> {
        self.pane_rows().iter().map(|row| row.key.clone()).collect()
    }

    pub(super) fn marks_gone_from_the_reading(&self) -> Vec<String> {
        let marked = self.marked();
        let Some(pane) = self.pane() else {
            return marked;
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return marked;
        };

        marked
            .into_iter()
            .filter(|key| !snapshot.items.contains_key(key))
            .collect()
    }

    pub(super) fn pane_detail(&self) -> Rc<Vec<Piece>> {
        let (Some(showing), Some(pane)) = (self.showing_pane(), self.pane()) else {
            return Rc::default();
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Rc::default();
        };
        let rows = self.pane_rows();
        let Some(row) = rows.get(showing.cursor) else {
            return Rc::default();
        };
        let width = self.look.text_width(self.body.get().width);
        let question = (
            self.view.readings.generation(),
            self.nav.at(),
            showing.at,
            row.clone(),
            width,
        );

        self.detail_seen
            .get_or(question, || Rc::new(pane.detail(snapshot, row, width)))
    }

    pub(super) fn pane_caption(&self) -> String {
        self.pane()
            .map_or_else(String::new, |pane| pane.caption().to_string())
    }

    pub(super) fn pane_detail_caption(&self) -> &'static str {
        self.pane().map_or("", |pane| pane.detail_caption())
    }

    pub(super) fn pane_sortable(&self) -> Vec<&'static str> {
        let Some(pane) = self.pane() else {
            return Vec::new();
        };
        if !pane.offers().sorting {
            return Vec::new();
        }

        let mut columns = vec![crate::ui::AS_READ];
        columns.extend(pane.sorted_by());
        columns
    }
}

fn arranged(pane: &dyn Pane, chosen: Option<&str>) -> Option<&'static str> {
    let every = pane.arrangements();
    let first = every.first()?.name;

    match chosen {
        Some(chosen) => every
            .iter()
            .find(|one| one.name == chosen)
            .map(|one| one.name)
            .or(Some(first)),
        None => Some(first),
    }
}
