use serde_json::{Value, json};
use vigil_model::Snapshot;
use vigil_view::{Index, Pane, Showing, listed};

use super::host::a_host_with_names_that_prefix_one_another as host;
use crate::fixture::users;
use crate::types::Subject;
use crate::views::pane::Of;

const COPIES: usize = 12;

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

fn searches_for(index: &Index) -> Vec<String> {
    let mut searches: Vec<String> = vec![
        String::new(),
        "nothing on this host reads like this".to_string(),
    ];
    searches.extend(('a'..='z').chain('0'..='9').map(String::from));
    searches.extend(
        [
            "|",
            ".",
            "%",
            "deploy",
            "deploy.1",
            "DEPLOY2",
            "SHA256:",
            "unreadable",
            "utmp",
            "Pts/",
        ]
        .map(String::from),
    );
    for at in (0..index.len()).step_by((index.len() / 6).max(1)) {
        let text: Vec<char> = index.haystack(at).chars().collect();
        for (from, length) in [(0, 2), (text.len() / 3, 5), (text.len() / 2, 9)] {
            let piece: String = text.iter().skip(from).take(length).collect();
            searches.push(piece.to_uppercase());
            searches.push(piece);
        }
    }
    searches
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
        let pane = Of(subject);
        let index = pane
            .index(&reading, &Showing::default())
            .expect("every accounts list builds an index");

        for search in searches_for(&index) {
            let showing = Showing::searching(&search);
            let (found, from_the_index) = listed(&pane, &reading, &showing, &index, None);
            assert_eq!(
                from_the_index,
                pane.rows(&reading, &showing),
                "{subject:?} searching {search:?}: the console answers a keystroke from the index, \
                 and the rows read from the reading are what it must say"
            );

            for longer in [
                format!("{search}e"),
                format!("{search}.1"),
                format!("{search}|"),
            ] {
                let narrowed = Showing::searching(&longer);
                assert_eq!(
                    listed(&pane, &reading, &narrowed, &index, Some(&found)).1,
                    pane.rows(&reading, &narrowed),
                    "{subject:?} narrowing {search:?} to {longer:?}: the next keystroke only looks \
                     among what the last one found, and must still find what the reading holds"
                );
            }
        }
    }
}
