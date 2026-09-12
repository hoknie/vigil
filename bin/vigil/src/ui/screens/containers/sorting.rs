use super::fields;
use super::row::Row;
use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[
    AS_READ,
    "CONTAINER",
    "PROGRAM",
    "RUNTIME",
    "SYS_ADMIN",
    "HOST PATHS",
];

pub fn sort(rows: &mut [Row<'_>], sorting: Sorting) {
    if sorting.as_read() {
        return;
    }
    rows.sort_by(|left, right| {
        let ordering = key(left, sorting.by).cmp(&key(right, sorting.by));
        match sorting.descending {
            true => ordering.reverse(),
            false => ordering,
        }
    });
}

fn key(row: &Row<'_>, by: usize) -> String {
    match by {
        1 => fields::what(row),
        2 => fields::program(row),
        3 => fields::runtime(row),
        4 => fields::may_take_the_host(row),
        5 => format!("{:0>6}", fields::mounted(row)),
        _ => String::new(),
    }
}
