use crate::ui::{Arrows, Program, Search};

pub struct Showing<'a> {
    pub program: Program,
    pub search: &'a Search,
    pub cursor: usize,
    pub elsewhere: usize,
    pub arrows: Arrows,
}
