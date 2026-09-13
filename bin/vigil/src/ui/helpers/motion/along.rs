use crate::ui::{Choice, Cursor, Search};

#[derive(Debug, Clone)]
pub struct Along<C: Choice> {
    at: C,
    cursors: Vec<Cursor>,
    searches: Vec<Search>,
}

impl<C: Choice> Default for Along<C> {
    fn default() -> Self {
        Along {
            at: C::default(),
            cursors: vec![Cursor::default(); C::COUNT],
            searches: vec![Search::default(); C::COUNT],
        }
    }
}

impl<C: Choice> Along<C> {
    pub fn showing(&self) -> C {
        self.at
    }

    pub fn show(&mut self, choice: C) {
        self.at = choice;
    }

    pub fn step_along(&mut self, by: isize, shown: &[C]) {
        self.at = self.at.step(by, shown);
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursors[self.at.index()]
    }

    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursors[self.at.index()]
    }

    pub fn at(&self) -> usize {
        self.cursor().at()
    }

    pub fn search(&self) -> &Search {
        &self.searches[self.at.index()]
    }

    pub fn search_mut(&mut self) -> &mut Search {
        &mut self.searches[self.at.index()]
    }

    pub fn narrowed_elsewhere(&self) -> usize {
        let here = self.at.index();
        self.searches
            .iter()
            .enumerate()
            .filter(|(index, search)| *index != here && search.holding_back())
            .count()
    }

    pub fn widen(&mut self) {
        self.searches[self.at.index()].clear();
        self.cursors[self.at.index()] = Cursor::default();
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::Startup;

    use super::*;

    fn looking_for(along: &mut Along<Startup>, word: &str) {
        let search = along.search_mut();
        search.start();
        for character in word.chars() {
            search.type_character(character);
        }
        search.accept();
    }

    #[test]
    fn a_search_belongs_to_the_list_it_was_typed_into_and_not_to_the_screen() {
        let mut along: Along<Startup> = Along::default();
        looking_for(&mut along, "docker");

        along.show(Startup::Timers);

        assert_eq!(along.search().query(), "");
        assert_eq!(
            along.narrowed_elsewhere(),
            1,
            "a list left narrowed has to be counted where the reader is looking"
        );
        along.show(Startup::Units);
        assert_eq!(along.search().query(), "docker");
    }

    #[test]
    fn widening_a_list_puts_its_cursor_back_at_the_top_of_it() {
        let mut along: Along<Startup> = Along::default();
        looking_for(&mut along, "root");
        along
            .cursor_mut()
            .step(crate::ui::Motion::Down, &["a".into(), "b".into()], 10);

        along.widen();

        assert!(!along.search().holding_back());
        assert_eq!(along.at(), 0);
    }
}
