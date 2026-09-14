use vigil_view::Facet;

use crate::ui::{Arrows, Gone, Search, Sorting};

pub struct Showing<'a> {
    pub at: usize,
    pub search: &'a Search,
    pub hidden: &'a [String],
    pub cursor: usize,
    pub arrows: Arrows,
    pub sorting: Sorting,
    pub note: Option<&'a str>,
    pub elsewhere: usize,
    pub gone: Option<&'a Gone>,
    pub arranged: Option<&'a str>,
    pub marked: Vec<String>,
    pub opened: Vec<String>,
    pub only: &'a [Facet],
}

impl Showing<'_> {
    pub fn hidden(&self) -> Vec<&str> {
        self.hidden.iter().map(String::as_str).collect()
    }

    pub fn opened(&self) -> Vec<&str> {
        self.opened.iter().map(String::as_str).collect()
    }

    pub fn is_marked(&self, key: &str) -> bool {
        self.marked.iter().any(|marked| marked == key)
    }
}

pub fn asked<'a>(
    showing: &'a Showing<'a>,
    hidden: &'a [&'a str],
    opened: &'a [&'a str],
) -> vigil_view::Showing<'a> {
    vigil_view::Showing {
        search: showing.search.query(),
        hidden,
        sorting: vigil_view::Sorting {
            by: showing.sorting.by,
            descending: showing.sorting.descending,
        },
        note: showing.note,
        elsewhere: showing.elsewhere,
        arranged: showing.arranged,
        opened,
        only: showing.only,
    }
}
