use crate::ui::{Dismissed, Filter, Picked, Sorting};

pub struct Showing<'a> {
    pub filter: &'a Filter,
    pub cursor: usize,
    pub focused: bool,
    pub sorting: Sorting,
    pub picked: &'a Picked,
    pub dismissed: &'a Dismissed,
}
