use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;

use vigil_view::{Assembled, Index, Placed};

use super::Shown;
use crate::ui::types::focus::position::keyed::Keyed;

const NOT_LISTED: usize = usize::MAX;

pub struct Listed {
    shown: Rc<Shown>,
    places: OnceCell<Vec<usize>>,
    headings: OnceCell<Vec<usize>>,
    keys: OnceCell<HashMap<String, usize>>,
}

impl Listed {
    pub fn of(shown: Rc<Shown>) -> Listed {
        Listed {
            shown,
            places: OnceCell::new(),
            headings: OnceCell::new(),
            keys: OnceCell::new(),
        }
    }

    fn row_placed(
        &self,
        index: &Index,
        key: &str,
        numbered: impl Iterator<Item = (usize, usize)>,
    ) -> Option<usize> {
        let places = self.places.get_or_init(|| {
            let mut places = vec![NOT_LISTED; index.len()];
            for (place, at) in numbered {
                places[at] = places[at].min(place);
            }
            places
        });
        index
            .keyed(key)
            .iter()
            .map(|at| places[*at])
            .filter(|place| *place != NOT_LISTED)
            .min()
    }

    fn heading_placed(&self, placed: &[Placed], key: &str) -> Option<usize> {
        let rows = self.shown.rows();
        let headings = self.headings.get_or_init(|| {
            let mut headings: Vec<usize> = placed
                .iter()
                .enumerate()
                .filter(|(_, placed)| matches!(placed, Placed::Heading { .. }))
                .map(|(place, _)| place)
                .collect();
            headings.sort_by(|left, right| {
                rows.key(*left).cmp(&rows.key(*right)).then(left.cmp(right))
            });
            headings
        });
        let from =
            headings.partition_point(|place| rows.key(*place).is_some_and(|said| said < key));
        headings
            .get(from)
            .filter(|place| rows.key(**place) == Some(key))
            .copied()
    }
}

impl Keyed for Listed {
    fn count(&self) -> usize {
        self.shown.rows().len()
    }

    fn key_at(&self, at: usize) -> Option<&str> {
        self.shown.rows().key(at)
    }

    fn position_of(&self, key: &str) -> Option<usize> {
        match (self.shown.index(), self.shown.assembled()) {
            (Some(index), Assembled::Ordered(numbered)) => {
                self.row_placed(index, key, numbered.iter().copied().enumerate())
            }
            (Some(index), Assembled::Gathered(placed)) => {
                let socket = self.row_placed(
                    index,
                    key,
                    placed
                        .iter()
                        .enumerate()
                        .filter_map(|(place, placed)| match placed {
                            Placed::Row(at) => Some((place, *at)),
                            Placed::Heading { .. } => None,
                        }),
                );
                [socket, self.heading_placed(placed, key)]
                    .into_iter()
                    .flatten()
                    .min()
            }
            _ => self
                .keys
                .get_or_init(|| {
                    let rows = self.shown.rows();
                    (0..rows.len())
                        .rev()
                        .filter_map(|at| rows.key(at).map(|key| (key.to_string(), at)))
                        .collect()
                })
                .get(key)
                .copied(),
        }
    }
}

#[cfg(test)]
mod tests {
    use vigil_view::RowKey;

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

    fn agrees(listed: &Listed, keys: &[&str]) {
        for key in keys.iter().copied().chain(["nothing like it"]) {
            assert_eq!(
                listed.position_of(key),
                walked(listed, key),
                "{key}: the cursor finds the row it held by its key on every press, a key shown \
                 twice is found where it is first shown, and a list that keeps the numbers of \
                 its rows is read from the index and not from copies of the rows"
            );
        }
    }

    #[test]
    fn a_list_says_where_a_key_is_as_reading_it_from_the_top_would_without_reading_it() {
        let names = ["heading", "a", "b", "a", "c"];
        let rows: Vec<RowKey> = names.iter().map(|name| RowKey::of(*name)).collect();
        let listed = Listed::of(Rc::new(Shown::built(rows)));

        assert_eq!(listed.count(), names.len());
        agrees(&listed, &names);
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
        agrees(&listed, &["heading", "a", "b", "c"]);
    }

    #[test]
    fn a_tree_of_row_numbers_says_where_a_heading_or_a_row_under_it_is() {
        let mut index = Index::new(0);
        for (key, heading) in [
            ("tcp|:80", "program|/usr/sbin/nginx"),
            ("tcp|:443", "program|/usr/sbin/nginx"),
            ("udp|:53", "unresolved"),
            ("tcp|:22", "program|/usr/sbin/sshd"),
        ] {
            index.push(
                RowKey::of(key).under(1).beneath(heading),
                key,
                0,
                Vec::new(),
            );
        }
        let listed = Listed::of(Rc::new(Shown::indexed(
            Rc::new(index),
            Assembled::Gathered(vec![
                Placed::Heading {
                    first: 0,
                    gathers: 2,
                    opened: true,
                },
                Placed::Row(1),
                Placed::Row(0),
                Placed::Heading {
                    first: 3,
                    gathers: 1,
                    opened: false,
                },
                Placed::Heading {
                    first: 2,
                    gathers: 1,
                    opened: true,
                },
                Placed::Row(2),
            ]),
        )));

        assert_eq!(listed.count(), 6);
        agrees(
            &listed,
            &[
                "program|/usr/sbin/nginx",
                "tcp|:80",
                "tcp|:443",
                "program|/usr/sbin/sshd",
                "tcp|:22",
                "unresolved",
                "udp|:53",
            ],
        );
    }
}
