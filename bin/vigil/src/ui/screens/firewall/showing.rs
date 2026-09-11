use crate::ui::{Arrows, Gone, Search};

pub struct Showing<'a> {
    pub search: &'a Search,
    pub cursor: usize,
    pub arrows: Arrows,
    pub gone: Option<&'a Gone>,
}
