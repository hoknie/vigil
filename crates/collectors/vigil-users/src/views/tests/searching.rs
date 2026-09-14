use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::{Pane, Showing, conformance};

use super::host::a_host_with_names_that_prefix_one_another as host;
use crate::fixture::users;
use crate::types::Subject;
use crate::views::pane::Of;

const COPIES: usize = 12;

pub(super) const SEARCHED: &[&str] = &[
    "deploy",
    "deploy.1",
    "DEPLOY2",
    "SHA256:",
    "sha256:",
    "unreadable",
    "utmp",
    "logind",
    "Pts/",
    "pts/",
    "session-source",
];

fn renamed(key: &str, item: &Value, copy: usize) -> (String, Value) {
    let suffix = format!(".{copy}");

    let mut parts: Vec<String> = key.split('|').map(str::to_string).collect();
    if let Some(named) = parts.get_mut(1) {
        named.push_str(&suffix);
    }

    let mut item = item.clone();
    for field in ["name", "user", "who", "source"] {
        if let Some(said) = item.get(field).and_then(Value::as_str) {
            item[field] = json!(format!("{said}{suffix}"));
        }
    }
    for field in ["members", "groups"] {
        if let Some(listed) = item.get_mut(field).and_then(Value::as_array_mut) {
            for entry in listed {
                if let Some(said) = entry.as_str() {
                    *entry = json!(format!("{said}{suffix}"));
                }
            }
        }
    }
    (parts.join("|"), item)
}

pub(super) fn scaled() -> Snapshot {
    let mut base = host();
    for (key, item) in users().items {
        base.items.entry(key).or_insert(item);
    }

    let mut scaled = base.clone();
    scaled.items.clear();
    for copy in 1..=COPIES {
        for (key, item) in &base.items {
            let (key, item) = renamed(key, item, copy);
            scaled.items.insert(key, item);
        }
    }
    scaled
}

#[test]
fn the_scaled_reading_holds_every_row_a_search_from_the_index_could_get_wrong() {
    let reading = scaled();

    for key in [
        "account|deploy.1",
        "account|deploy.10",
        "account|deploy2.1",
        "sshkey|dep.1|unreadable",
        "session-source|utmp.1",
        "keyring|root.1|0x1234",
    ] {
        assert!(
            reading.items.contains_key(key),
            "{key}: without it one of the rows an index can list differently from the reading is \
             never searched for"
        );
    }
    for subject in Subject::ALL.iter().copied() {
        let pane = Of(subject);
        let index = pane
            .index(&reading, &Showing::default())
            .expect("every accounts list builds an index");

        assert!(
            !index.is_empty(),
            "{subject:?}: an empty index agrees with the rows about everything and proves nothing"
        );
        assert_eq!(
            index.len(),
            pane.rows(&reading, &Showing::default()).len(),
            "{subject:?}: the index holds every row the list shows before anything is typed"
        );
    }
}

#[test]
fn every_list_answers_each_keystroke_from_its_index_exactly_as_its_rows_answer_it() {
    let reading = scaled();

    for subject in Subject::ALL.iter().copied() {
        conformance::the_index_lists_every_search_and_sort_as_the_rows_do_also(
            &Of(subject),
            &reading,
            Showing::default(),
            SEARCHED,
        );
    }
}
