use crate::ui::{Arrows, Search, Startup};

pub struct Showing<'a> {
    pub list: Startup,
    pub search: &'a Search,
    pub cursor: usize,
    pub elsewhere: usize,
    pub arrows: Arrows,
}
