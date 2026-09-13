use super::sorting::Sorting;
use crate::helpers::haystack;

use serde_json::Value;

#[derive(Debug, Clone, Copy, Default)]
pub struct Showing<'a> {
    pub search: &'a str,
    pub hidden: &'a [&'a str],
    pub sorting: Sorting,
    pub note: Option<&'a str>,
    pub elsewhere: usize,
}

impl<'a> Showing<'a> {
    pub fn searching(search: &'a str) -> Showing<'a> {
        Showing {
            search,
            ..Showing::default()
        }
    }

    pub fn hiding(self, hidden: &'a [&'a str]) -> Showing<'a> {
        Showing { hidden, ..self }
    }

    pub fn sorted(self, sorting: Sorting) -> Showing<'a> {
        Showing { sorting, ..self }
    }

    pub fn noting(self, note: Option<&'a str>) -> Showing<'a> {
        Showing { note, ..self }
    }

    pub fn wants(&self, toggle: &str) -> bool {
        !self.hidden.contains(&toggle)
    }

    pub fn holding_back(&self) -> bool {
        !self.search.is_empty()
    }

    pub fn narrowed(&self) -> bool {
        self.holding_back() || !self.hidden.is_empty()
    }

    pub fn matches(&self, key: &str, item: &Value) -> bool {
        if self.search.is_empty() {
            return true;
        }
        haystack(key, item)
            .to_lowercase()
            .contains(&self.search.to_lowercase())
    }
}
