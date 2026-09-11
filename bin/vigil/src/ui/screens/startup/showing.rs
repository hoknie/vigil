use crate::ui::{Arrows, Nesting, Search, Startup};

pub struct Showing<'a> {
    pub list: Startup,
    pub search: &'a Search,
    pub nesting: Nesting,
    pub cursor: usize,
    pub elsewhere: usize,
    pub arrows: Arrows,
}

impl<'a> Showing<'a> {
    pub fn plain(list: Startup, search: &'a Search) -> Showing<'a> {
        Showing {
            list,
            search,
            nesting: Nesting::default(),
            cursor: 0,
            elsewhere: 0,
            arrows: Arrows::Away,
        }
    }

    pub fn as_a_tree(&self) -> bool {
        self.list == Startup::Units && self.nesting.nested()
    }
}
