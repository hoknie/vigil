use std::rc::Rc;

use vigil_view::{Assembled, Index, RowKey, Rows};

pub struct Shown {
    index: Option<Rc<Index>>,
    assembled: Assembled,
}

impl Shown {
    pub fn indexed(index: Rc<Index>, assembled: Assembled) -> Shown {
        Shown {
            index: Some(index),
            assembled,
        }
    }

    pub fn built(rows: Vec<RowKey>) -> Shown {
        Shown {
            index: None,
            assembled: Assembled::Built(rows),
        }
    }

    pub fn rows(&self) -> Rows<'_> {
        match (&self.index, &self.assembled) {
            (Some(index), assembled) => Rows::of(index, assembled),
            (None, Assembled::Built(rows)) => Rows::built(rows),
            (None, Assembled::Ordered(_)) => Rows::built(&[]),
        }
    }

    pub fn numbered(&self) -> Option<(&Index, &[usize])> {
        match (&self.index, &self.assembled) {
            (Some(index), Assembled::Ordered(at)) => Some((index, at)),
            _ => None,
        }
    }
}
