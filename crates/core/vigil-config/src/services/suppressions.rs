use crate::helpers::quoting::unquoted;
use crate::types::entry::Entry;

const HEADING: &str = "suppressions:";

const EMPTY: &str = "suppressions: []";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Changed { text: String, entries: usize },
    AlreadySo,
    NotOurs(String),
}

pub fn add(text: &str, entries: &[Entry]) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let (heading, held) = match shape(&lines) {
        Shape::NotOurs => return Edit::NotOurs(shape_we_do_not_edit()),
        Shape::Missing => (None, Vec::new()),
        Shape::Empty(at) => (Some(at), Vec::new()),
        Shape::Block(at) => (Some(at), items(&lines, at)),
    };

    let mut named: Vec<Named> = held.iter().filter_map(Item::named).collect();
    let mut written: Vec<String> = Vec::new();
    let mut wrote = 0;
    for entry in entries {
        let asked = Named {
            key: entry.key.clone(),
            prefix: entry.prefix,
            kind: entry.kind.clone(),
        };
        if named.contains(&asked) {
            continue;
        }
        named.push(asked);
        written.extend(entry.lines());
        wrote += 1;
    }
    if written.is_empty() {
        return Edit::AlreadySo;
    }

    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();
    match heading {
        None => {
            while out.last().is_some_and(|line| line.trim().is_empty()) {
                out.pop();
            }
            out.push(String::new());
            out.push(HEADING.to_string());
            out.extend(written);
        }
        Some(at) => {
            out[at] = HEADING.to_string();
            let after = held.last().map(|item| item.end).unwrap_or(at + 1);
            for (moved, line) in written.into_iter().enumerate() {
                out.insert(after + moved, line);
            }
        }
    }
    Edit::Changed {
        text: joined(&out),
        entries: wrote,
    }
}

pub fn remove(text: &str, keys: &[String]) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let heading = match shape(&lines) {
        Shape::NotOurs => return Edit::NotOurs(shape_we_do_not_edit()),
        Shape::Missing | Shape::Empty(_) => return Edit::AlreadySo,
        Shape::Block(at) => at,
    };

    let held = items(&lines, heading);
    let doomed: Vec<&Item> = held
        .iter()
        .filter(|item| {
            item.key
                .as_ref()
                .is_some_and(|(key, _)| keys.iter().any(|wanted| wanted == key))
        })
        .collect();
    if doomed.is_empty() {
        return Edit::AlreadySo;
    }

    let mut out: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(at, _)| !doomed.iter().any(|item| item.holds(*at)))
        .map(|(_, line)| (*line).to_string())
        .collect();
    if held.len() == doomed.len() {
        out[heading] = EMPTY.to_string();
    }
    Edit::Changed {
        text: joined(&out),
        entries: doomed.len(),
    }
}

pub fn named(text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    match shape(&lines) {
        Shape::Block(at) => items(&lines, at)
            .into_iter()
            .filter_map(|item| item.key.map(|(key, _)| key))
            .collect(),
        _ => Vec::new(),
    }
}

enum Shape {
    Missing,
    Empty(usize),
    Block(usize),
    NotOurs,
}

struct Item {
    start: usize,
    end: usize,
    key: Option<(String, bool)>,
    kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Named {
    key: String,
    prefix: bool,
    kind: Option<String>,
}

impl Item {
    fn holds(&self, line: usize) -> bool {
        line >= self.start && line < self.end
    }

    fn named(&self) -> Option<Named> {
        let (key, prefix) = self.key.clone()?;

        Some(Named {
            key,
            prefix,
            kind: self.kind.clone(),
        })
    }
}

fn shape(lines: &[&str]) -> Shape {
    for (at, line) in lines.iter().enumerate() {
        if !line.starts_with(HEADING) {
            continue;
        }
        return match line[HEADING.len()..].trim() {
            "" => Shape::Block(at),
            "[]" => Shape::Empty(at),
            _ => Shape::NotOurs,
        };
    }
    Shape::Missing
}

fn items(lines: &[&str], heading: usize) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();

    for (at, line) in lines.iter().enumerate().skip(heading + 1) {
        let held = line.trim();
        if held.is_empty() || held.starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        match held.strip_prefix('-') {
            Some(first) => items.push(Item {
                start: at,
                end: at + 1,
                key: keyed(first.trim()),
                kind: kinded(first.trim()),
            }),
            None => {
                if let Some(item) = items.last_mut() {
                    item.end = at + 1;
                    if item.key.is_none() {
                        item.key = keyed(held);
                    }
                    if item.kind.is_none() {
                        item.kind = kinded(held);
                    }
                }
            }
        }
    }
    items
}

fn kinded(field: &str) -> Option<String> {
    field
        .strip_prefix("kind:")
        .map(|value| unquoted(value.trim()))
}

fn keyed(field: &str) -> Option<(String, bool)> {
    for (name, prefix) in [("finding_key_prefix:", true), ("finding_key:", false)] {
        if let Some(value) = field.strip_prefix(name) {
            return Some((unquoted(value.trim()), prefix));
        }
    }
    None
}

fn joined(lines: &[String]) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn shape_we_do_not_edit() -> String {
    format!(
        "the {HEADING} in this file is not written as a block of lines, and this command edits \
         no shape it did not write. Add the entry by hand, or write the file again with `vigild \
         configure --force`"
    )
}
