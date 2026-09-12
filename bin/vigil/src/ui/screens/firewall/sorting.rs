use super::fields;
use super::row::Row;
use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[AS_READ, "KIND", "WHAT", "HOOK", "POLICY", "RULES"];

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
        2 => fields::what(row).to_lowercase(),
        3 => fields::hook(row),
        4 => fields::policy(row),
        5 => format!("{:0>12}", fields::rules(row)),
        _ => String::new(),
    }
}
