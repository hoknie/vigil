use super::facet::Facet;
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
    pub arranged: Option<&'a str>,
    pub opened: &'a [&'a str],
    pub only: &'a [Facet],
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

    pub fn arranged(self, arranged: &'a str) -> Showing<'a> {
        Showing {
            arranged: Some(arranged),
            ..self
        }
    }

    pub fn opened_up(&self, heading: &str) -> bool {
        self.opened.contains(&heading)
    }

    pub fn opening(self, opened: &'a [&'a str]) -> Showing<'a> {
        Showing { opened, ..self }
    }

    pub fn narrowing(self, only: &'a [Facet]) -> Showing<'a> {
        Showing { only, ..self }
    }

    pub fn only(&self, facet: &str) -> Option<&'a str> {
        self.only
            .iter()
            .find(|chosen| chosen.name == facet)
            .map(|chosen| chosen.value.as_str())
    }

    pub fn arranged_as(&self, name: &str) -> bool {
        self.arranged == Some(name)
    }

    pub fn wants(&self, toggle: &str) -> bool {
        !self.hidden.contains(&toggle)
    }

    pub fn holding_back(&self) -> bool {
        !self.search.is_empty() || !self.only.is_empty()
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
