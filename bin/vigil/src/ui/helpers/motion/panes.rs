use std::collections::BTreeSet;

use vigil_view::Facet;

use crate::ui::{Cursor, Search};

#[derive(Debug, Clone)]
pub struct Panes {
    at: usize,
    cursors: Vec<Cursor>,
    searches: Vec<Search>,
    marked: Vec<BTreeSet<String>>,
    opened: Vec<BTreeSet<String>>,
    only: Vec<Vec<Facet>>,
    hidden: Vec<String>,
    arranged: Option<String>,
}

impl Panes {
    pub fn of(count: usize) -> Panes {
        let count = count.max(1);
        Panes {
            at: 0,
            cursors: vec![Cursor::default(); count],
            searches: vec![Search::default(); count],
            marked: vec![BTreeSet::new(); count],
            opened: vec![BTreeSet::new(); count],
            only: vec![Vec::new(); count],
            hidden: Vec::new(),
            arranged: None,
        }
    }

    pub fn ready(&mut self, count: usize) {
        let count = count.max(1);
        self.cursors.resize(count, Cursor::default());
        self.searches.resize(count, Search::default());
        self.only.resize(count, Vec::new());
        self.at = self.at.min(count - 1);
    }

    pub fn showing(&self) -> usize {
        self.at
    }

    pub fn show(&mut self, at: usize) {
        self.at = at.min(self.cursors.len() - 1);
    }

    pub fn step_along(&mut self, by: isize, shown: &[usize]) {
        if shown.is_empty() {
            return;
        }
        let here = shown
            .iter()
            .position(|index| *index == self.at)
            .unwrap_or(0) as isize;
        let count = shown.len() as isize;
        self.at = shown[(here + by).rem_euclid(count) as usize];
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursors[self.at]
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[self.at]
    }

    pub fn at(&self) -> usize {
        self.cursor().at()
    }

    pub fn search(&self) -> &Search {
        &self.searches[self.at]
    }

    pub fn search_mut(&mut self) -> &mut Search {
        &mut self.searches[self.at]
    }

    pub fn marked(&self) -> Vec<String> {
        self.marked[self.at].iter().cloned().collect()
    }

    pub fn is_marked(&self, key: &str) -> bool {
        self.marked[self.at].contains(key)
    }

    pub fn mark(&mut self, key: &str, wanted: bool) {
        match wanted {
            true => {
                self.marked[self.at].insert(key.to_string());
            }
            false => {
                self.marked[self.at].remove(key);
            }
        }
    }

    pub fn unmark_everything(&mut self) {
        self.marked[self.at].clear();
    }

    pub fn forget_marks(&mut self, gone: &[String]) {
        for key in gone {
            self.marked[self.at].remove(key);
        }
    }

    pub fn opened(&self) -> Vec<String> {
        self.opened[self.at].iter().cloned().collect()
    }

    pub fn open(&mut self, key: &str, wanted: bool) {
        match wanted {
            true => {
                self.opened[self.at].insert(key.to_string());
            }
            false => {
                self.opened[self.at].remove(key);
            }
        }
    }

    pub fn hidden(&self) -> &[String] {
        &self.hidden
    }

    pub fn hiding(&self) -> bool {
        !self.hidden.is_empty()
    }

    pub fn toggle(&mut self, name: &str) {
        match self.hidden.iter().position(|hidden| hidden == name) {
            Some(at) => {
                self.hidden.remove(at);
            }
            None => self.hidden.push(name.to_string()),
        }
    }

    pub fn arranged(&self) -> Option<&str> {
        self.arranged.as_deref()
    }

    pub fn arrange(&mut self, name: &str) {
        self.arranged = Some(name.to_string());
    }

    pub fn show_every_kind(&mut self) {
        self.hidden.clear();
    }

    pub fn only(&self) -> &[Facet] {
        &self.only[self.at]
    }

    pub fn narrow_to(&mut self, facet: Facet) {
        let chosen = &mut self.only[self.at];
        chosen.retain(|one| one.name != facet.name);
        chosen.push(facet);
    }

    pub fn widen_facet(&mut self, name: &str) {
        self.only[self.at].retain(|one| one.name != name);
    }

    pub fn narrow_to_nothing(&mut self) {
        self.only[self.at].clear();
    }

    pub fn narrowed_elsewhere(&self) -> usize {
        self.searches
            .iter()
            .enumerate()
            .filter(|(index, search)| *index != self.at && search.holding_back())
            .count()
    }

    pub fn widen(&mut self) {
        self.searches[self.at].clear();
        self.cursors[self.at] = Cursor::default();
    }
}
