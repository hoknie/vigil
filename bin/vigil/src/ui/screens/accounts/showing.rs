use crate::ui::{Arrows, Search, Subject};

pub struct Showing<'a> {
    pub subject: Subject,
    pub search: &'a Search,
    pub cursor: usize,
    pub elsewhere: usize,
    pub arrows: Arrows,
}
