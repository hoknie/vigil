use super::{Assembled, Index};
use crate::types::RowKey;

#[derive(Clone, Copy)]
pub enum Rows<'a> {
    Ordered { index: &'a Index, at: &'a [usize] },
    Built(&'a [RowKey]),
}

impl<'a> Rows<'a> {
    pub fn of(index: &'a Index, assembled: &'a Assembled) -> Rows<'a> {
        match assembled {
            Assembled::Ordered(at) => Rows::Ordered { index, at },
            Assembled::Built(rows) => Rows::Built(rows),
        }
    }

    pub fn built(rows: &'a [RowKey]) -> Rows<'a> {
        Rows::Built(rows)
    }

    pub fn len(&self) -> usize {
        match self {
            Rows::Ordered { at, .. } => at.len(),
            Rows::Built(rows) => rows.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, place: usize) -> Option<&'a RowKey> {
        match *self {
            Rows::Ordered { index, at } => at.get(place).map(|at| index.row(*at)),
            Rows::Built(rows) => rows.get(place),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &'a RowKey> + 'a> {
        match *self {
            Rows::Ordered { index, at } => Box::new(at.iter().map(move |at| index.row(*at))),
            Rows::Built(rows) => Box::new(rows.iter()),
        }
    }

    pub fn to_vec(&self) -> Vec<RowKey> {
        self.iter().cloned().collect()
    }
}

impl<'a> IntoIterator for &Rows<'a> {
    type Item = &'a RowKey;
    type IntoIter = Box<dyn Iterator<Item = &'a RowKey> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
