use crate::ui::{Cursor, Search};

#[derive(Debug, Clone)]
pub struct Panes {
    at: usize,
    cursors: Vec<Cursor>,
    searches: Vec<Search>,
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
            hidden: Vec::new(),
            arranged: None,
        }
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
}
