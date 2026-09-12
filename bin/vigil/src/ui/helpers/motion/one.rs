use crate::ui::{Cursor, Search};

#[derive(Debug, Clone, Default)]
pub struct One {
    cursor: Cursor,
    search: Search,
}

impl One {
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }

    pub fn at(&self) -> usize {
        self.cursor.at()
    }

    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn search_mut(&mut self) -> &mut Search {
        &mut self.search
    }

    pub fn widen(&mut self) {
        self.search.clear();
        self.cursor = Cursor::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_screen_with_one_list_holds_a_cursor_and_a_search_of_its_own() {
        let mut one = One::default();
        one.search_mut().start();
        one.search_mut().type_character('x');
        one.search_mut().accept();
        one.cursor_mut()
            .step(crate::ui::Motion::Down, &["a".into(), "b".into()], 10);

        assert!(one.search().holding_back());
        assert_eq!(one.at(), 1);

        one.widen();

        assert!(!one.search().holding_back());
        assert_eq!(
            one.at(),
            0,
            "widening a list puts its cursor back at the top"
        );
    }
}
