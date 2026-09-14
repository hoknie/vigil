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

    pub fn forget_marks_not_in(&mut self, rows: &[String]) {
        self.marked[self.at].retain(|key| rows.iter().any(|row| row == key));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_a_list_is_narrowed_to_replaces_the_one_of_its_name_and_stays_with_that_list() {
        let mut panes = Panes::of(2);
        panes.narrow_to(Facet::new("user", "alice"));
        panes.narrow_to(Facet::new("program", "/usr/bin/nc"));
        panes.narrow_to(Facet::new("user", "root"));

        assert_eq!(
            panes.only(),
            &[
                Facet::new("program", "/usr/bin/nc"),
                Facet::new("user", "root")
            ],
            "a list narrowed to alice and then to root is a list of root's rows, not of \
             nobody's"
        );

        panes.show(1);
        assert!(panes.only().is_empty());

        panes.show(0);
        panes.widen_facet("user");
        assert_eq!(panes.only(), &[Facet::new("program", "/usr/bin/nc")]);
        panes.narrow_to_nothing();
        assert!(panes.only().is_empty());
    }

    #[test]
    fn a_section_that_grows_a_list_can_narrow_the_new_one_without_falling_off_the_end() {
        let mut panes = Panes::of(1);

        panes.ready(2);
        panes.show(1);
        panes.narrow_to(Facet::new("user", "alice"));

        assert_eq!(panes.only().len(), 1);
    }

    #[test]
    fn a_search_belongs_to_the_pane_it_was_typed_into_and_not_to_the_section() {
        let mut panes = Panes::of(2);
        let search = panes.search_mut();
        search.start();
        search.type_character('n');
        search.accept();

        panes.show(1);

        assert_eq!(panes.search().query(), "");
        assert_eq!(panes.narrowed_elsewhere(), 1);
        panes.show(0);
        assert_eq!(panes.search().query(), "n");
    }

    #[test]
    fn the_row_of_names_wraps_rather_than_stopping_at_its_ends() {
        let mut panes = Panes::of(2);

        panes.step_along(-1, &[0, 1]);
        assert_eq!(
            panes.showing(),
            1,
            "a reader stepping left off the first list lands on the last, which is what \
             every other row of names in this console does"
        );
        panes.step_along(1, &[0, 1]);
        assert_eq!(panes.showing(), 0);
    }

    #[test]
    fn a_pane_that_is_not_shown_is_stepped_over_rather_than_landed_on() {
        let mut panes = Panes::of(3);

        panes.step_along(1, &[0, 2]);

        assert_eq!(
            panes.showing(),
            2,
            "a section whose last pane has nothing in it must not strand the reader on it"
        );
    }

    #[test]
    fn what_is_marked_belongs_to_the_list_it_was_marked_in() {
        let mut panes = Panes::of(2);
        panes.mark("tcp|0.0.0.0:4444", true);

        panes.show(1);
        assert_eq!(
            panes.marked().len(),
            0,
            "the two lists show the same sockets under different keys, and carrying a mark \
             across would aim the kill at a row the reader never saw"
        );

        panes.show(0);
        assert!(panes.is_marked("tcp|0.0.0.0:4444"));
        panes.mark("tcp|0.0.0.0:4444", false);
        assert!(panes.marked().is_empty());
    }

    #[test]
    fn a_mark_on_a_socket_that_is_no_longer_in_the_reading_is_dropped_rather_than_kept() {
        let mut panes = Panes::of(1);
        panes.mark("tcp|0.0.0.0:4444", true);
        panes.mark("tcp|0.0.0.0:22", true);

        panes.forget_marks_not_in(&["tcp|0.0.0.0:22".to_string()]);

        assert_eq!(
            panes.marked(),
            vec!["tcp|0.0.0.0:22".to_string()],
            "the port closed under the reader while it was marked; carrying the mark to the \
             next reading is how a kill lands on whatever takes that port next"
        );
    }

    #[test]
    fn a_branch_opened_in_one_list_is_not_opened_in_the_other() {
        let mut panes = Panes::of(2);
        panes.open("program|/usr/sbin/nginx", true);

        assert_eq!(panes.opened(), vec!["program|/usr/sbin/nginx".to_string()]);
        panes.show(1);
        assert!(panes.opened().is_empty());
    }

    #[test]
    fn a_kind_switched_off_and_on_again_leaves_nothing_hidden() {
        let mut panes = Panes::of(1);

        panes.toggle("udp");
        assert!(panes.hiding());
        panes.toggle("udp");
        assert!(!panes.hiding());

        panes.toggle("tcp");
        panes.show_every_kind();
        assert!(!panes.hiding());
    }

    #[test]
    fn a_section_whose_lists_the_agent_decides_keeps_a_cursor_for_each_of_them() {
        let mut panes = Panes::of(1);

        panes.ready(3);
        panes.show(2);
        panes.cursor_mut().point_at("b", &["a".into(), "b".into()]);
        assert_eq!(panes.at(), 1);

        panes.ready(1);
        assert_eq!(
            panes.showing(),
            0,
            "an agent that stopped sending a reading must not leave the reader pointing at a \
             list that is no longer there"
        );
    }
}
