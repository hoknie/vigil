use std::collections::BTreeMap;
use std::sync::OnceLock;

use memchr::memmem::Finder;

use crate::Facet;
use crate::types::{RowKey, Sorting};

const UNGATHERED: usize = usize::MAX;

const BETWEEN_ROWS: char = '\0';

const CHARACTERS_KEPT: usize = 128;

pub struct Index {
    rows: Vec<RowKey>,
    text: String,
    ends: Vec<usize>,
    groups: Vec<u8>,
    keys: Vec<Vec<String>>,
    columns: usize,
    facets: BTreeMap<&'static str, BTreeMap<String, Vec<usize>>>,
    faceted: bool,
    by_key: OnceLock<Vec<usize>>,
    singles: [OnceLock<Vec<usize>>; CHARACTERS_KEPT],
    headings: Vec<usize>,
    ranks: Vec<[OnceLock<Vec<usize>>; 2]>,
}

impl Index {
    pub fn new(columns: usize) -> Index {
        Index {
            rows: Vec::new(),
            text: String::new(),
            ends: Vec::new(),
            groups: Vec::new(),
            keys: Vec::new(),
            columns,
            facets: BTreeMap::new(),
            faceted: false,
            by_key: OnceLock::new(),
            singles: std::array::from_fn(|_| OnceLock::new()),
            headings: Vec::new(),
            ranks: (0..columns)
                .map(|_| [OnceLock::new(), OnceLock::new()])
                .collect(),
        }
    }

    pub fn push(&mut self, row: RowKey, haystack: &str, group: u8, mut keys: Vec<String>) {
        let start = self.text.len();
        match haystack.is_ascii() {
            true => {
                self.text.push_str(haystack);
                self.text[start..].make_ascii_lowercase();
            }
            false => self.text.push_str(&haystack.to_lowercase()),
        }
        self.ends.push(self.text.len());
        self.text.push(BETWEEN_ROWS);
        keys.resize(self.columns, String::new());

        self.rows.push(row);
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
        let start = match at.checked_sub(1) {
            Some(before) => self.ends[before] + BETWEEN_ROWS.len_utf8(),
            None => 0,
        };
        &self.text[start..self.ends[at]]
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
        let finder = Finder::new(query.as_bytes());
        let holds = |at: &usize| finder.find(self.haystack(*at).as_bytes()).is_some();
        if let Some(within) = within {
            return within.iter().copied().filter(holds).collect();
        }
        if query.contains(BETWEEN_ROWS) {
            return (0..self.rows.len()).filter(holds).collect();
        }
        if let [byte] = query.as_bytes()
            && let Some(kept) = self.singles.get(usize::from(*byte))
        {
            return kept.get_or_init(|| self.scanned(&finder)).clone();
        }
        match self.among(&query) {
            Some(among) => among.iter().copied().filter(holds).collect(),
            None => self.scanned(&finder),
        }
    }

    fn among(&self, query: &str) -> Option<&[usize]> {
        let fewest = query
            .bytes()
            .filter_map(|byte| self.singles.get(usize::from(byte)))
            .filter_map(OnceLock::get)
            .min_by_key(|rows| rows.len());
        if let Some(fewest) = fewest {
            return Some(fewest);
        }
        let (byte, kept) = query
            .bytes()
            .find_map(|byte| Some((byte, self.singles.get(usize::from(byte))?)))?;
        Some(kept.get_or_init(|| self.scanned(&Finder::new(std::slice::from_ref(&byte)))))
    }

    fn scanned(&self, finder: &Finder<'_>) -> Vec<usize> {
        let text = self.text.as_bytes();
        let length = finder.needle().len();
        let mut found = Vec::new();
        let mut from = 0;
        while let Some(offset) = finder.find(&text[from..]) {
            let at = self
                .ends
                .partition_point(|end| *end < from + offset + length);
            found.push(at);
            match self.ends.get(at) {
                Some(end) => from = end + BETWEEN_ROWS.len_utf8(),
                None => break,
            }
        }
        found
    }

    #[cfg(test)]
    pub(super) fn keeps(&self, character: char) -> bool {
        u8::try_from(character)
            .ok()
            .and_then(|byte| self.singles.get(usize::from(byte)))
            .is_some_and(|kept| kept.get().is_some())
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
