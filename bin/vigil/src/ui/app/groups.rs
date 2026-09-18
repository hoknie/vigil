use vigil_view::Section;

use super::App;

use crate::ui::{Level, Offset};

impl App {
    pub(in crate::ui::app) fn groups(&self) -> Vec<&'static str> {
        self.section()
            .map(|section| section.groups())
            .unwrap_or_default()
    }

    pub(in crate::ui::app) fn group_showing(&self) -> Option<&'static str> {
        match self.section() {
            Some(section) => self.group_of(section.as_ref()),
            None => None,
        }
    }

    pub(in crate::ui::app) fn group_of(&self, section: &dyn Section) -> Option<&'static str> {
        let named = section.groups();
        let chosen = self.panes().and_then(crate::ui::Panes::group);

        named
            .iter()
            .find(|group| Some(**group) == chosen)
            .or_else(|| named.first())
            .copied()
    }

    pub(in crate::ui::app) fn group_of_the_pane(&self, at: usize) -> Option<&'static str> {
        self.section()?.panes().get(at)?.belongs_to()
    }

    pub(in crate::ui::app) fn step_the_groups(&mut self, by: isize) {
        let named = self.groups();
        if named.is_empty() {
            return;
        }
        let here = self
            .group_showing()
            .and_then(|group| named.iter().position(|named| *named == group))
            .unwrap_or(0) as isize;
        let next = named[(here + by).rem_euclid(named.len() as isize) as usize];

        self.show_the_group(next);
    }

    pub(in crate::ui::app) fn end_of_the_groups(&mut self, last: bool) {
        let named = self.groups();
        let at = match last {
            true => named.len().saturating_sub(1),
            false => 0,
        };
        if let Some(group) = named.get(at).copied() {
            self.show_the_group(group);
        }
    }

    pub(in crate::ui::app) fn show_the_group(&mut self, named: &'static str) {
        if let Some(panes) = self.panes_mut() {
            panes.choose_group(named);
        }
        if let Some(first) = self.shown_panes().first().copied()
            && let Some(panes) = self.panes_mut()
        {
            panes.show(first);
        }
        self.detail_open = false;
        self.nav.difference = Offset::default();
        self.refresh_wanted = true;
        self.settle();
    }

    pub(in crate::ui::app) fn press_the_group(&mut self, at: usize) {
        let Some(group) = self.groups().get(at).copied() else {
            return;
        };

        self.show_the_group(group);
        self.level = Level::top(self.rungs());
    }
}
