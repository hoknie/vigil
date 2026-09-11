use super::arrangement::Arrangement;
use crate::ui::{Arrows, Protocols, Search};

pub struct Showing<'a> {
    pub arrangement: Arrangement,
    pub protocols: &'a Protocols,
    pub search: &'a Search,
    pub cursor: usize,
    pub arrows: Arrows,
}
