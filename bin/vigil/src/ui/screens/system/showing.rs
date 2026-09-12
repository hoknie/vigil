use crate::ui::{Arrows, Gone, Search, Sorting, System};

pub struct Showing<'a> {
    pub showing: System,
    pub search: &'a Search,
    pub cursor: usize,
    pub elsewhere: usize,
    pub arrows: Arrows,
    pub gone: Option<&'a Gone>,
    pub sorting: Sorting,
}
