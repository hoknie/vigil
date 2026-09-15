use std::borrow::Cow;

use super::{Assembled, Index, Placed};
use crate::types::RowKey;

#[derive(Clone, Copy)]
pub enum Rows<'a> {
    Ordered {
        index: &'a Index,
        at: &'a [usize],
    },
    Gathered {
        index: &'a Index,
        placed: &'a [Placed],
    },
    Built(&'a [RowKey]),
}

impl<'a> Rows<'a> {
    pub fn of(index: &'a Index, assembled: &'a Assembled) -> Rows<'a> {
        match assembled {
            Assembled::Ordered(at) => Rows::Ordered { index, at },
            Assembled::Gathered(placed) => Rows::Gathered { index, placed },
            Assembled::Built(rows) => Rows::Built(rows),
        }
    }

    pub fn built(rows: &'a [RowKey]) -> Rows<'a> {
        Rows::Built(rows)
    }

    pub fn len(&self) -> usize {
        match self {
            Rows::Ordered { at, .. } => at.len(),
            Rows::Gathered { placed, .. } => placed.len(),
            Rows::Built(rows) => rows.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn key(&self, place: usize) -> Option<&'a str> {
        match *self {
            Rows::Ordered { index, at } => at.get(place).map(|at| index.row(*at).key.as_str()),
            Rows::Gathered { index, placed } => placed.get(place).map(|placed| match *placed {
                Placed::Row(at) => index.row(at).key.as_str(),
                Placed::Heading { first, .. } => heading_of(index.row(first)),
            }),
            Rows::Built(rows) => rows.get(place).map(|row| row.key.as_str()),
        }
    }

    pub fn get(&self, place: usize) -> Option<Cow<'a, RowKey>> {
        match *self {
            Rows::Ordered { index, at } => at.get(place).map(|at| Cow::Borrowed(index.row(*at))),
            Rows::Gathered { index, placed } => {
                placed.get(place).map(|placed| placed_row(index, *placed))
            }
            Rows::Built(rows) => rows.get(place).map(Cow::Borrowed),
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = Cow<'a, RowKey>> + 'a> {
        let rows = *self;
        Box::new((0..rows.len()).filter_map(move |place| rows.get(place)))
    }

    pub fn to_vec(&self) -> Vec<RowKey> {
        self.iter().map(Cow::into_owned).collect()
    }
}

impl<'a> IntoIterator for &Rows<'a> {
    type Item = Cow<'a, RowKey>;
    type IntoIter = Box<dyn Iterator<Item = Cow<'a, RowKey>> + 'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

fn heading_of(first: &RowKey) -> &str {
    first
        .gathered_under
        .as_deref()
        .unwrap_or(first.key.as_str())
}

fn placed_row(index: &Index, placed: Placed) -> Cow<'_, RowKey> {
    match placed {
        Placed::Row(at) => Cow::Borrowed(index.row(at)),
        Placed::Heading {
            first,
            gathers,
            opened,
        } => {
            let first = index.row(first);
            let mut heading = RowKey::of(heading_of(first))
                .of_its_own()
                .gathering(gathers)
                .opened(opened);
            heading.named = first.named.clone();
            Cow::Owned(heading)
        }
    }
}
