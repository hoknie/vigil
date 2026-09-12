#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choosing {
    Sort,
    Filter,
}

impl Choosing {
    pub fn caption(self) -> &'static str {
        match self {
            Choosing::Sort => "sort by",
            Choosing::Filter => "show only",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Chooser {
    open: Option<Choosing>,
    at: usize,
    offered: Vec<String>,
}

impl Chooser {
    pub fn open(&mut self, what: Choosing, offered: Vec<String>, at: usize) {
        self.at = at.min(offered.len().saturating_sub(1));
        self.offered = offered;
        self.open = Some(what);
    }

    pub fn close(&mut self) {
        self.open = None;
        self.offered.clear();
        self.at = 0;
    }

    pub fn choosing(&self) -> Option<Choosing> {
        self.open
    }

    pub fn at(&self) -> usize {
        self.at
    }

    pub fn offered(&self) -> &[String] {
        &self.offered
    }

    pub fn step(&mut self, by: isize) {
        if self.offered.is_empty() {
            return;
        }
        let count = self.offered.len() as isize;
        self.at = (self.at as isize + by).rem_euclid(count) as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn offered() -> Vec<String> {
        vec!["as read".into(), "TIME ↑".into(), "TIME ↓".into()]
    }

    #[test]
    fn a_console_nobody_has_pressed_a_key_on_is_choosing_nothing() {
        let chooser = Chooser::default();

        assert_eq!(chooser.choosing(), None);
        assert!(chooser.offered().is_empty());
    }

    #[test]
    fn it_opens_on_what_is_chosen_now_rather_than_at_the_top_of_the_list() {
        let mut chooser = Chooser::default();

        chooser.open(Choosing::Sort, offered(), 2);

        assert_eq!(chooser.choosing(), Some(Choosing::Sort));
        assert_eq!(chooser.at(), 2, "a reader looks for where they are, first");
    }

    #[test]
    fn the_arrows_walk_the_options_and_come_round_rather_than_stopping() {
        let mut chooser = Chooser::default();
        chooser.open(Choosing::Sort, offered(), 0);

        chooser.step(-1);
        assert_eq!(chooser.at(), 2);
        chooser.step(1);
        assert_eq!(chooser.at(), 0);
    }

    #[test]
    fn closing_it_leaves_nothing_behind_for_the_next_screen_to_draw() {
        let mut chooser = Chooser::default();
        chooser.open(Choosing::Filter, offered(), 1);

        chooser.close();

        assert_eq!(chooser.choosing(), None);
        assert_eq!(chooser.at(), 0);
        assert!(chooser.offered().is_empty());
    }

    #[test]
    fn an_option_past_the_end_is_not_a_cursor_off_the_list() {
        let mut chooser = Chooser::default();

        chooser.open(Choosing::Sort, offered(), 99);

        assert_eq!(chooser.at(), 2);
    }
}
