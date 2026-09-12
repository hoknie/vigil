use crate::ui::{Arrows, Gone, Search, Sorting};

pub struct Showing<'a> {
    pub search: &'a Search,
    pub cursor: usize,
    pub arrows: Arrows,
    pub gone: Option<&'a Gone>,
    pub sorting: Sorting,
}
