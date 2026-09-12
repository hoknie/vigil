use crate::ui::{Filter, Sorting};

pub struct Showing<'a> {
    pub filter: &'a Filter,
    pub cursor: usize,
    pub focused: bool,
    pub sorting: Sorting,
}
