use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Shown;
use crate::ui::types::focus::position::keyed::Keyed;

const NOT_LISTED: usize = usize::MAX;

pub struct Listed {
    shown: Rc<Shown>,
    places: OnceCell<Vec<usize>>,
    keys: OnceCell<HashMap<String, usize>>,
}

impl Listed {
    pub fn of(shown: Rc<Shown>) -> Listed {
        Listed {
            shown,
            places: OnceCell::new(),
            keys: OnceCell::new(),
        }
    }
}

impl Keyed for Listed {
    fn count(&self) -> usize {
        self.shown.rows().len()
    }

    fn key_at(&self, at: usize) -> Option<&str> {
        self.shown.rows().get(at).map(|row| row.key.as_str())
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        if let Some((index, numbered)) = self.shown.numbered() {
            let places = self.places.get_or_init(|| {
                let mut places = vec![NOT_LISTED; index.len()];
                for (place, at) in numbered.iter().enumerate() {
                    places[*at] = places[*at].min(place);
                }
                places
            });
            return index
                .keyed(key)
                .iter()
                .map(|at| places[*at])
                .filter(|place| *place != NOT_LISTED)
                .min();
        }

        self.keys
            .get_or_init(|| {
                let rows = self.shown.rows();
                (0..rows.len())
                    .rev()
                    .filter_map(|at| rows.get(at).map(|row| (row.key.clone(), at)))
                    .collect()
            })
            .get(key)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use vigil_view::{Assembled, Index, RowKey};

    use super::*;

    fn walked(listed: &Listed, key: &str) -> Option<usize> {
        let keys: Vec<String> = listed
            .shown
            .rows()
            .iter()
            .map(|row| row.key.clone())
            .collect();
        keys.position_of(key)
    }

    #[test]
    fn a_list_says_where_a_key_is_as_reading_it_from_the_top_would_without_reading_it() {
        let names = ["heading", "a", "b", "a", "c"];
        let rows: Vec<RowKey> = names.iter().map(|name| RowKey::of(*name)).collect();
        let listed = Listed::of(Rc::new(Shown::built(rows)));

        assert_eq!(listed.count(), names.len());
        for key in ["heading", "a", "b", "c", "nothing like it"] {
            assert_eq!(
                listed.position_of(key),
                walked(&listed, key),
                "{key}: the cursor finds the row it held by its key on every press, and a key \
                 shown twice is found where it is first shown"
            );
        }
        for at in 0..=names.len() {
            assert_eq!(
                listed.key_at(at),
                names.get(at).copied(),
                "the row at a place is the row drawn there"
            );
        }
    }

    #[test]
    fn a_list_of_row_numbers_says_where_a_key_is_as_reading_it_from_the_top_would() {
        let mut index = Index::new(0);
        for key in ["heading", "a", "b", "a", "c"] {
            index.push(RowKey::of(key), key, 1, Vec::new());
        }
        let listed = Listed::of(Rc::new(Shown::indexed(
            Rc::new(index),
            Assembled::Ordered(vec![3, 2, 1, 4]),
        )));

        assert_eq!(listed.count(), 4);
        for key in ["heading", "a", "b", "c", "nothing like it"] {
            assert_eq!(
                listed.position_of(key),
                walked(&listed, key),
                "{key}: a list the index answered keeps only the numbers of its rows, so the \
                 place of a key is read from the index and not from copies of the rows"
            );
        }
    }
}
