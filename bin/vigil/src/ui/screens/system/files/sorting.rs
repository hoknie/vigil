use super::fields;
use super::row::Row;
use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[AS_READ, "KIND", "PATH", "MODE", "OWNER", "SIZE", "STANDING"];

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
        3 => fields::mode(row),
        4 => fields::owner(row),
        5 => format!("{:0>20}", row.item["size"].as_u64().unwrap_or_default()),
        6 => fields::standing(row),
        _ => String::new(),
    }
}
