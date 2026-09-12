use super::fields;
use super::row::Row;
use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[AS_READ, "KIND", "WHAT", "FREE", "INODES", "SIZE"];

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
        1 => row.kind.name().to_string(),
        2 => fields::what(row),
        3 => padded(fields::free_step(row.item)),
        4 => padded(fields::free_inodes_step(row.item)),
        5 => format!("{:0>20}", fields::number(row.item, "total_bytes")),
        _ => String::new(),
    }
}

fn padded(step: Option<u64>) -> String {
    match step {
        Some(step) => format!("{step:0>3}"),
        None => String::new(),
    }
}
