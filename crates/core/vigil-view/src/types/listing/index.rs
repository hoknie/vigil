use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::Facet;
use crate::types::{RowKey, Sorting};

const UNGATHERED: usize = usize::MAX;

pub struct Index {
    rows: Vec<RowKey>,
    haystacks: Vec<String>,
    groups: Vec<u8>,
    keys: Vec<Vec<String>>,
    columns: usize,
    postings: BTreeMap<char, Vec<usize>>,
    facets: BTreeMap<&'static str, BTreeMap<String, Vec<usize>>>,
    faceted: bool,
    by_key: OnceLock<Vec<usize>>,
    headings: Vec<usize>,
    ranks: Vec<[OnceLock<Vec<usize>>; 2]>,
}

impl Index {
    pub fn new(columns: usize) -> Index {
        Index {
            rows: Vec::new(),
            haystacks: Vec::new(),
            groups: Vec::new(),
            keys: Vec::new(),
            columns,
            postings: BTreeMap::new(),
            facets: BTreeMap::new(),
            faceted: false,
            by_key: OnceLock::new(),
            headings: Vec::new(),
            ranks: (0..columns)
                .map(|_| [OnceLock::new(), OnceLock::new()])
                .collect(),
        }
    }

    pub fn push(&mut self, row: RowKey, haystack: &str, group: u8, mut keys: Vec<String>) {
        let at = self.rows.len();
        let lowered = haystack.to_lowercase();
        let mut characters: Vec<char> = lowered.chars().collect();
        characters.sort_unstable();
        characters.dedup();
        for character in characters {
            self.postings.entry(character).or_default().push(at);
        }
        keys.resize(self.columns, String::new());

        self.rows.push(row);
        self.haystacks.push(lowered);
        self.groups.push(group);
        self.keys.push(keys);
        self.headings.push(UNGATHERED);
    }

    pub fn faceted(&mut self, facets: Vec<Facet>) {
        self.faceted = true;
        let Some(at) = self.rows.len().checked_sub(1) else {
            return;
        };
        for facet in facets {
            let postings = self
                .facets
                .entry(facet.name)
                .or_default()
                .entry(facet.value)
                .or_default();
            if postings.last() != Some(&at) {
                postings.push(at);
            }
        }
    }

    pub fn narrowed(&self, only: &[Facet]) -> Option<Vec<usize>> {
        if !self.faceted {
            return None;
        }
        let mut chosen: Vec<&[usize]> = only
            .iter()
            .enumerate()
            .filter(|(place, facet)| {
                !only[..*place]
                    .iter()
                    .any(|before| before.name == facet.name)
            })
            .map(|(_, facet)| {
                self.facets
                    .get(facet.name)
                    .and_then(|values| values.get(&facet.value))
                    .map_or(&[][..], Vec::as_slice)
            })
            .collect();
        chosen.sort_by_key(|postings| postings.len());
        let (fewest, others) = chosen.split_first()?;

        Some(
            fewest
                .iter()
                .copied()
                .filter(|at| others.iter().all(|other| other.binary_search(at).is_ok()))
                .collect(),
        )
    }

    pub fn gathered(&mut self, heading: usize) {
        if let Some(last) = self.headings.last_mut() {
            *last = heading;
        }
    }

    pub fn heading_of(&self, at: usize) -> Option<usize> {
        self.headings
            .get(at)
            .copied()
            .filter(|heading| *heading != UNGATHERED)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn row(&self, at: usize) -> &RowKey {
        &self.rows[at]
    }

    pub fn haystack(&self, at: usize) -> &str {
        &self.haystacks[at]
    }

    pub fn keyed(&self, key: &str) -> &[usize] {
        let by_key = self.by_key.get_or_init(|| {
            let mut order: Vec<usize> = (0..self.rows.len()).collect();
            order.sort_by(|left, right| {
                self.rows[*left]
                    .key
                    .cmp(&self.rows[*right].key)
                    .then(left.cmp(right))
            });
            order
        });
        let from = by_key.partition_point(|at| self.rows[*at].key.as_str() < key);
        let to = by_key.partition_point(|at| self.rows[*at].key.as_str() <= key);
        &by_key[from..to]
    }

    pub fn found(&self, search: &str, within: Option<&[usize]>) -> Vec<usize> {
        if search.is_empty() {
            return (0..self.rows.len()).collect();
        }
        let query = search.to_lowercase();
        let rarest: &[usize] = query
            .chars()
            .map(|character| self.postings.get(&character).map_or(&[][..], Vec::as_slice))
            .min_by_key(|posting| posting.len())
            .unwrap_or(&[]);
        let candidates = match within {
            Some(within) if within.len() < rarest.len() => within,
            _ => rarest,
        };

        candidates
            .iter()
            .copied()
            .filter(|at| self.haystacks[*at].contains(&query))
            .collect()
    }

    pub fn ordered(&self, mut found: Vec<usize>, sorting: Sorting) -> Vec<usize> {
        if sorting.as_read() || sorting.by > self.columns {
            return found;
        }
        let rank = self.rank(sorting.by - 1, sorting.descending);
        found.sort_unstable_by_key(|at| rank[*at]);
        found
    }

    fn rank(&self, column: usize, descending: bool) -> &[usize] {
        self.ranks[column][usize::from(descending)].get_or_init(|| {
            let mut order: Vec<usize> = (0..self.rows.len()).collect();
            order.sort_by(|left, right| {
                let by = self.keys[*left][column].cmp(&self.keys[*right][column]);
                let by = match descending {
                    true => by.reverse(),
                    false => by,
                };
                self.groups[*left].cmp(&self.groups[*right]).then(by)
            });
            let mut rank = vec![0; order.len()];
            for (place, at) in order.into_iter().enumerate() {
                rank[at] = place;
            }
            rank
        })
    }
}
