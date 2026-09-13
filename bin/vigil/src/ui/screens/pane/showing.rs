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
}

impl Showing<'_> {
    pub fn hidden(&self) -> Vec<&str> {
        self.hidden.iter().map(String::as_str).collect()
    }
}

pub fn asked<'a>(showing: &'a Showing<'a>, hidden: &'a [&'a str]) -> vigil_view::Showing<'a> {
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
    }
}
