use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use vigil_view::RowKey;

use crate::ui::types::focus::position::keyed::Keyed;

pub struct Listed {
    rows: Rc<Vec<RowKey>>,
    positions: OnceCell<HashMap<String, usize>>,
}

impl Listed {
    pub fn of(rows: Rc<Vec<RowKey>>) -> Listed {
        Listed {
            rows,
            positions: OnceCell::new(),
        }
    }
}

impl Keyed for Listed {
    fn count(&self) -> usize {
        self.rows.len()
    }

    fn key_at(&self, at: usize) -> Option<&str> {
        self.rows.get(at).map(|row| row.key.as_str())
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        self.positions
            .get_or_init(|| {
                self.rows
                    .iter()
                    .enumerate()
                    .rev()
                    .map(|(at, row)| (row.key.clone(), at))
                    .collect()
            })
            .get(key)
            .copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_says_where_a_key_is_as_reading_it_from_the_top_would_without_reading_it() {
        let names = ["heading", "a", "b", "a", "c"];
        let rows: Vec<RowKey> = names.iter().map(|name| RowKey::of(*name)).collect();
        let keys: Vec<String> = names.iter().map(|name| (*name).to_string()).collect();
        let listed = Listed::of(Rc::new(rows));

        assert_eq!(listed.count(), keys.count());
        for key in ["heading", "a", "b", "c", "nothing like it"] {
            assert_eq!(
                listed.position_of(key),
                keys.position_of(key),
                "{key}: the cursor finds the row it held by its key on every press, and a key \
                 shown twice is found where it is first shown"
            );
        }
        for at in 0..=names.len() {
            assert_eq!(listed.key_at(at), keys.key_at(at));
        }
    }
}
