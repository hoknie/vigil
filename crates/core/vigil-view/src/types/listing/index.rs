use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::types::{RowKey, Sorting};

pub struct Index {
    rows: Vec<RowKey>,
    haystacks: Vec<String>,
    groups: Vec<u8>,
    keys: Vec<Vec<String>>,
    columns: usize,
    postings: BTreeMap<char, Vec<usize>>,
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
