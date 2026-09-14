use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    held: BTreeSet<String>,
    run: Option<Run>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Run {
    from: String,
    before: BTreeSet<String>,
}

impl Default for Picked {
    fn default() -> Self {
        Picked::none()
    }
}

impl Picked {
    pub const fn none() -> Picked {
        Picked {
            held: BTreeSet::new(),
            run: None,
        }
    }

    pub fn holds(&self, key: &str) -> bool {
        self.held.contains(key)
    }

    pub fn count(&self) -> usize {
        self.held.len()
    }

    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    pub fn toggle(&mut self, key: &str) {
        self.run = None;
        if !self.held.remove(key) {
            self.held.insert(key.to_string());
        }
    }

    pub fn every(&mut self, keys: &[String]) {
        self.run = None;
        match !keys.is_empty() && keys.iter().all(|key| self.held.contains(key)) {
            true => self.held.clear(),
            false => self.held.extend(keys.iter().cloned()),
        }
    }

    pub fn opening(&mut self, keys: &[String], at: usize) {
        if self.run.is_some() {
            return;
        }
        let Some(from) = keys.get(at) else {
            return;
        };
        self.run = Some(Run {
            from: from.clone(),
            before: self.held.clone(),
        });
    }

    pub fn reaching(&mut self, keys: &[String], at: usize) {
        let Some(run) = self.run.as_ref() else {
            return;
        };
        let from = keys
            .iter()
            .position(|key| key == &run.from)
            .unwrap_or(at.min(keys.len().saturating_sub(1)));
        let (first, last) = (from.min(at), from.max(at));
        let Some(reached) = keys.get(first..=last) else {
            return;
        };
        let mut held = run.before.clone();
        held.extend(reached.iter().cloned());
        self.held = held;
    }

    pub fn let_go(&mut self) {
        self.run = None;
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.run = None;
    }

    pub fn settle(&mut self, keys: &[String]) {
        if self.held.is_empty() {
            self.run = None;
            return;
        }
        let shown: BTreeSet<&str> = keys.iter().map(String::as_str).collect();
        self.held.retain(|key| shown.contains(key.as_str()));
        if self
            .run
            .as_ref()
            .is_some_and(|run| !shown.contains(run.from.as_str()))
        {
            self.run = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    fn rows() -> Vec<String> {
        keys(&["a", "b", "c", "d", "e"])
    }

    fn held(picked: &Picked) -> Vec<String> {
        rows().into_iter().filter(|row| picked.holds(row)).collect()
    }

    fn run(picked: &mut Picked, rows: &[String], from: usize, over: &[usize]) {
        picked.opening(rows, from);
        for at in over {
            picked.reaching(rows, *at);
        }
    }

    #[test]
    fn a_console_nobody_has_picked_anything_on_holds_nothing() {
        let picked = Picked::default();

        assert!(picked.is_empty());
        assert_eq!(picked.count(), 0);
        assert!(!picked.holds("a"));
    }

    #[test]
    fn a_run_holds_every_row_between_where_it_began_and_where_it_is_now() {
        let rows = rows();
        let mut picked = Picked::default();

        run(&mut picked, &rows, 1, &[2, 3]);

        assert_eq!(held(&picked), keys(&["b", "c", "d"]));
    }

    #[test]
    fn a_run_walked_back_over_lets_go_of_the_rows_it_passed_rather_than_keeping_them() {
        let rows = rows();
        let mut picked = Picked::default();

        run(&mut picked, &rows, 1, &[2, 3, 2, 1]);

        assert_eq!(
            held(&picked),
            keys(&["b"]),
            "a reader who overshot has no way back if the run only ever grows"
        );
    }

    #[test]
    fn a_run_that_turns_round_past_where_it_began_picks_upwards_from_there() {
        let rows = rows();
        let mut picked = Picked::default();

        run(&mut picked, &rows, 2, &[3, 2, 1, 0]);

        assert_eq!(held(&picked), keys(&["a", "b", "c"]));
    }

    #[test]
    fn a_second_run_keeps_what_the_first_one_picked() {
        let rows = rows();
        let mut picked = Picked::default();
        run(&mut picked, &rows, 0, &[1]);

        picked.let_go();
        run(&mut picked, &rows, 3, &[4]);

        assert_eq!(held(&picked), keys(&["a", "b", "d", "e"]));
    }

    #[test]
    fn one_row_at_a_time_goes_on_and_comes_off_again_with_the_same_key() {
        let mut picked = Picked::default();

        picked.toggle("c");
        assert!(picked.holds("c"));

        picked.toggle("c");
        assert!(!picked.holds("c"));
    }

    #[test]
    fn every_row_at_once_is_a_switch_and_not_a_one_way_door() {
        let rows = rows();
        let mut picked = Picked::default();

        picked.every(&rows);
        assert_eq!(picked.count(), 5);

        picked.every(&rows);
        assert!(
            picked.is_empty(),
            "a reader who picked everything by mistake presses the same key to undo it"
        );
    }

    #[test]
    fn picking_every_row_of_a_list_with_nothing_in_it_picks_nothing() {
        let mut picked = Picked::default();

        picked.every(&[]);

        assert!(picked.is_empty());
    }

    #[test]
    fn a_row_that_left_the_screen_is_no_longer_picked_and_cannot_be_acted_on_unseen() {
        let mut picked = Picked::default();
        picked.toggle("b");
        picked.toggle("z");

        picked.settle(&rows());

        assert_eq!(
            held(&picked),
            keys(&["b"]),
            "the agent keeps the last so many findings, and a deed must not reach one that \
             rolled off the list under the reader"
        );
    }

    #[test]
    fn a_run_whose_beginning_rolled_off_the_list_starts_again_where_the_cursor_is() {
        let rows = rows();
        let mut picked = Picked::default();
        picked.opening(&rows, 0);
        picked.settle(&keys(&["c", "d", "e"]));

        picked.reaching(&keys(&["c", "d", "e"]), 1);

        assert!(picked.is_empty() || picked.holds("d"));
    }

    #[test]
    fn letting_go_of_a_run_keeps_the_rows_and_only_forgets_where_it_began() {
        let rows = rows();
        let mut picked = Picked::default();
        run(&mut picked, &rows, 0, &[2]);

        picked.let_go();

        assert_eq!(picked.count(), 3);
    }

    #[test]
    fn clearing_leaves_nothing_behind_for_the_next_deed_to_reach() {
        let rows = rows();
        let mut picked = Picked::default();
        run(&mut picked, &rows, 0, &[4]);

        picked.clear();

        assert!(picked.is_empty());
        assert_eq!(picked, Picked::default());
    }
}
