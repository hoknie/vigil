use super::fields::marked;
use super::row::Row;
use crate::ui::helpers::words::haystack;
use crate::ui::{Program, Reading, Search, View};

pub fn rows<'a>(view: &'a View, program: Program, search: &Search) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading(program.collector()) else {
        return Vec::new();
    };

    let mut rows: Vec<Row<'a>> = snapshot
        .items
        .iter()
        .filter(|(key, item)| search.matches(&haystack::haystack(key, item)))
        .map(|(key, item)| Row {
            key: key.clone(),
            item,
            mark: marked(key),
        })
        .collect();

    rows.sort_by_key(|row| (!row.mark, row.key.clone()));
    rows
}

pub fn keys(view: &View, program: Program, search: &Search) -> Vec<String> {
    rows(view, program, search)
        .into_iter()
        .map(|row| row.key)
        .collect()
}

pub fn objects(view: &View, program: Program) -> usize {
    rows(view, program, &Search::default())
        .into_iter()
        .filter(|row| !row.mark)
        .count()
}

pub fn marks(view: &View, program: Program) -> usize {
    rows(view, program, &Search::default())
        .into_iter()
        .filter(|row| row.mark)
        .count()
}
