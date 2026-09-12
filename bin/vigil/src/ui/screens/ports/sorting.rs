use super::fields::{basename, endpoint, protocol, user};
use super::row::Row;
use super::what::What;
use crate::ui::{AS_READ, Sorting};

pub const SORTED_BY: &[&str] = &[AS_READ, "PROTO", "ADDRESS", "USER", "PROGRAM"];

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
    let What::Socket(item) = &row.what else {
        return String::new();
    };
    match by {
        1 => protocol(item, &row.key).to_string(),
        2 => endpoint(item, &row.key),
        3 => user(item),
        4 => item
            .get("process")
            .and_then(|process| process.get("exe"))
            .and_then(serde_json::Value::as_str)
            .map(basename)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}
