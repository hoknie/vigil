use std::cmp::Ordering;

use serde_json::Value;
use vigil_view::{Column, Notice, Showing, Sorting, Width, basename, time_of_day};

use super::fields::{marked, number, text};

pub(super) const USER: &str = "user";

pub(super) const PROGRAM: &str = "program";

pub(super) fn columns(wide: bool) -> Vec<Column> {
    let mut columns = vec![
        Column::new("WHO", Width::Fixed(12)),
        Column::new("PROGRAM", Width::Least(14)),
        Column::new("RUNS", Width::Fixed(7)),
        Column::new("FIRST SEEN", Width::Share(2)),
    ];
    if wide {
        columns.push(Column::new("PATH", Width::Share(3)));
    }
    columns
}

pub(super) fn cells(key: &str, item: &Value, wide: bool) -> Vec<String> {
    let mut cells = match marked(key) {
        true => vec![
            "—".to_string(),
            "—".to_string(),
            "—".to_string(),
            text(item, "reason")
                .unwrap_or("this reading is not complete")
                .to_string(),
        ],
        false => vec![
            who(item),
            basename(executable(item)).to_string(),
            runs(item).to_string(),
            text(item, "first_seen")
                .map(time_of_day)
                .unwrap_or("?")
                .to_string(),
        ],
    };
    if wide {
        cells.push(match marked(key) {
            true => "—".to_string(),
            false => executable(item).to_string(),
        });
    }

    cells
}

pub(super) fn who(item: &Value) -> String {
    match text(item, "user") {
        Some(user) => user.to_string(),
        None => match item.get("auid").and_then(Value::as_u64) {
            Some(auid) => format!("login {auid}"),
            None => "?".to_string(),
        },
    }
}

pub(super) fn executable(item: &Value) -> &str {
    text(item, "exe").unwrap_or("?")
}

pub(super) fn runs(item: &Value) -> u64 {
    number(item, "runs").unwrap_or(1)
}

pub(super) fn chosen(key: &str, item: &Value, showing: &Showing<'_>) -> bool {
    if showing.only.is_empty() {
        return true;
    }
    if marked(key) {
        return false;
    }

    showing.only(USER).is_none_or(|user| who(item) == user)
        && showing
            .only(PROGRAM)
            .is_none_or(|program| executable(item) == program)
}

pub(super) fn order(left: (&str, &Value), right: (&str, &Value), sorting: Sorting) -> Ordering {
    let (left_key, left_item) = left;
    let (right_key, right_item) = right;

    let about_the_reading_first = marked(right_key).cmp(&marked(left_key));
    let by = match sorting.by {
        1 => who(left_item).cmp(&who(right_item)),
        2 => program(left_item).cmp(&program(right_item)),
        3 => runs(left_item).cmp(&runs(right_item)),
        4 => text(left_item, "first_seen").cmp(&text(right_item, "first_seen")),
        _ => Ordering::Equal,
    };
    let by = match sorting.descending {
        true => by.reverse(),
        false => by,
    };

    about_the_reading_first
        .then(by)
        .then_with(|| left_key.cmp(right_key))
}

pub(super) fn program(item: &Value) -> (&str, &str) {
    (basename(executable(item)), executable(item))
}

pub(super) fn narrowed_to(showing: &Showing<'_>) -> Option<String> {
    let said: Vec<String> = showing
        .only
        .iter()
        .map(|facet| format!("{} {}", facet.name, facet.value))
        .collect();
    match said.is_empty() {
        true => None,
        false => Some(said.join(" and ")),
    }
}

pub(super) fn empty(showing: &Showing<'_>) -> Notice {
    if showing.search.is_empty()
        && let Some(narrowed) = narrowed_to(showing)
    {
        return Notice::plain(format!(
            "No launch is left once the list is narrowed to {narrowed}."
        ))
        .saying("Press f and choose everything to see the whole list again.");
    }
    if showing.holding_back() {
        return Notice::plain(format!("No launch matches {:?}.", showing.search)).saying(
            "The search covers every value recorded about the row, and belongs to this list \
             alone. Press / to change it, Esc to drop it.",
        );
    }

    match showing.note {
        Some(reason) => Notice::loud("Nothing here, and the records are not being read.")
            .saying(reason.to_string()),
        None => Notice::plain("Nothing has been run since this agent started watching.")
            .saying("This list only grows: a row is never taken off it."),
    }
}
