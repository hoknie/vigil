use serde_json::Value;

use super::arrangement::Arrangement;
use super::fields::{basename, protocol};
use super::row::Row;
use super::showing::Showing;
use super::what::What;
use crate::ui::helpers::words::haystack;
use crate::ui::{Reading, View};

pub fn rows<'a>(view: &'a View, showing: &Showing<'_>) -> Vec<Row<'a>> {
    let Reading::Taken(snapshot) = view.reading("ports") else {
        return Vec::new();
    };

    let passing: Vec<(&String, &Value)> = snapshot
        .items
        .iter()
        .filter(|(key, item)| showing.protocols.showing(protocol(item, key)))
        .filter(|(key, item)| showing.search.matches(&haystack::haystack(key, item)))
        .collect();

    match showing.arrangement {
        Arrangement::Flat => {
            let mut rows: Vec<Row<'a>> = passing
                .into_iter()
                .map(|(key, item)| Row {
                    key: key.clone(),
                    what: What::Socket(item),
                })
                .collect();
            super::sorting::sort(&mut rows, showing.sorting);
            rows
        }
        Arrangement::ByProgram => grouped(passing),
    }
}

fn grouped<'a>(passing: Vec<(&'a String, &'a Value)>) -> Vec<Row<'a>> {
    let mut programs: Vec<(String, Vec<(&String, &Value)>)> = Vec::new();
    let mut unresolved: Vec<(&String, &Value)> = Vec::new();

    for (key, item) in passing {
        match holder(item) {
            None => unresolved.push((key, item)),
            Some(path) => match programs.iter_mut().find(|(known, _)| *known == path) {
                Some((_, sockets)) => sockets.push((key, item)),
                None => programs.push((path, vec![(key, item)])),
            },
        }
    }
    programs.sort_by(|left, right| left.0.cmp(&right.0));

    let mut rows = Vec::new();
    for (path, sockets) in &programs {
        let name = basename(path);
        let ambiguous = programs
            .iter()
            .filter(|(other, _)| basename(other) == name)
            .count()
            > 1;
        rows.push(Row {
            key: format!("program|{path}"),
            what: What::Program {
                path: path.clone(),
                count: sockets.len(),
                ambiguous,
            },
        });
        for (key, item) in sockets {
            rows.push(Row {
                key: (*key).clone(),
                what: What::Socket(item),
            });
        }
    }
    if !unresolved.is_empty() {
        rows.push(Row {
            key: "unresolved".to_string(),
            what: What::Unresolved {
                count: unresolved.len(),
            },
        });
        for (key, item) in unresolved {
            rows.push(Row {
                key: key.clone(),
                what: What::Socket(item),
            });
        }
    }
    rows
}

fn holder(item: &Value) -> Option<String> {
    if item.get("owner_resolved").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    item.get("process")
        .and_then(|process| process.get("exe"))
        .and_then(Value::as_str)
        .map(str::to_string)
}
