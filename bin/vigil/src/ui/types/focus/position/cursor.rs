use super::keyed::Keyed;
use crate::ui::Motion;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cursor {
    at: usize,
    holding: Option<String>,
}

impl Cursor {
    pub fn at(&self) -> usize {
        self.at
    }

    pub fn step<K: Keyed + ?Sized>(&mut self, motion: Motion, keys: &K, page: usize) {
        let last = keys.count().saturating_sub(1);
        self.at = match motion.distance(page) {
            Some(distance) => self.at.saturating_add_signed(distance).min(last),
            None => match motion {
                Motion::First => 0,
                _ => last,
            },
        };
        self.hold(keys);
    }

    pub fn point_at<K: Keyed + ?Sized>(&mut self, key: &str, keys: &K) -> bool {
        match keys.position_of(key) {
            Some(index) => {
                self.at = index;
                self.holding = Some(key.to_string());
                true
            }
            None => false,
        }
    }

    pub fn settle<K: Keyed + ?Sized>(&mut self, keys: &K) {
        if let Some(holding) = &self.holding
            && let Some(index) = keys.position_of(holding)
        {
            self.at = index;
            return;
        }
        self.at = self.at.min(keys.count().saturating_sub(1));
        self.hold(keys);
    }

    fn hold<K: Keyed + ?Sized>(&mut self, keys: &K) {
        self.holding = keys.key_at(self.at).map(str::to_string);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    #[test]
    fn the_cursor_never_leaves_the_list_it_is_in() {
        let rows = keys(&["a", "b", "c"]);
        let mut cursor = Cursor::default();

        for _ in 0..10 {
            cursor.step(Motion::Down, &rows, 10);
        }
        assert_eq!(cursor.at(), 2);

        for _ in 0..10 {
            cursor.step(Motion::Up, &rows, 10);
        }
        assert_eq!(cursor.at(), 0);
    }

    #[test]
    fn a_row_that_moved_under_the_cursor_is_followed_rather_than_lost() {
        let mut cursor = Cursor::default();
        cursor.step(Motion::Down, &keys(&["a", "b", "c"]), 10);
        assert_eq!(cursor.at(), 1);

        cursor.settle(&keys(&["new", "a", "b", "c"]));

        assert_eq!(cursor.at(), 2, "still on b, which is now one row lower");
    }

    #[test]
    fn a_row_that_disappeared_leaves_the_cursor_where_it_was_and_not_past_the_end() {
        let mut cursor = Cursor::default();
        cursor.step(Motion::Last, &keys(&["a", "b", "c"]), 10);

        cursor.settle(&keys(&["a"]));

        assert_eq!(cursor.at(), 0);
    }

    #[test]
    fn a_jump_to_a_row_that_is_not_there_leaves_the_cursor_alone() {
        let rows = keys(&["a", "b", "c"]);
        let mut cursor = Cursor::default();
        cursor.step(Motion::Down, &rows, 10);

        assert!(!cursor.point_at("nothing like it", &rows));
        assert_eq!(cursor.at(), 1);

        assert!(cursor.point_at("c", &rows));
        assert_eq!(cursor.at(), 2);
    }

    #[test]
    fn a_page_moves_a_screenful_and_the_ends_go_all_the_way() {
        let rows: Vec<String> = (0..100).map(|number| number.to_string()).collect();
        let mut cursor = Cursor::default();

        cursor.step(Motion::PageDown, &rows, 20);
        assert_eq!(cursor.at(), 19);

        cursor.step(Motion::Last, &rows, 20);
        assert_eq!(cursor.at(), 99);

        cursor.step(Motion::First, &rows, 20);
        assert_eq!(cursor.at(), 0);
    }

    #[test]
    fn an_empty_list_has_nowhere_to_put_a_cursor_and_does_not_divide_by_it() {
        let mut cursor = Cursor::default();
        cursor.step(Motion::Down, &keys(&[]), 10);
        cursor.settle(&keys(&[]));
        assert_eq!(cursor.at(), 0);
    }
}
