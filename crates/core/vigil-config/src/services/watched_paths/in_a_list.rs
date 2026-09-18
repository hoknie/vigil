use super::lines::{Item, joined, opened};
use crate::helpers::quoting::{quoted, unquoted};
use crate::services::suppressions::Edit;
use crate::types::watch::Watch;

const LIST: &str = "files:";

const SIZE: &str = "max_file_size:";

const UNDER_THE_LIST: &str = "  ";

const UNITS: &[(&str, u64)] = &[
    ("gb", 1024 * 1024 * 1024),
    ("mb", 1024 * 1024),
    ("kb", 1024),
    ("b", 1),
    ("", 1),
];

pub fn listed(text: &str) -> Vec<Watch> {
    let lines: Vec<&str> = text.lines().collect();
    match shape(&lines) {
        Shape::Block(at) => items(&lines, at)
            .into_iter()
            .map(|item| item.watch)
            .collect(),
        _ => Vec::new(),
    }
}

pub fn put_listed(text: &str, entry: &Watch) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();

    match shape(&lines) {
        Shape::NotOurs(why) => Edit::NotOurs(why),
        Shape::Missing => {
            while out.last().is_some_and(|line| line.trim().is_empty()) {
                out.pop();
            }
            if !out.is_empty() {
                out.push(String::new());
            }
            out.push(LIST.to_string());
            out.extend(written(entry, UNDER_THE_LIST));
            changed(&out)
        }
        Shape::Empty(at) => {
            out[at] = out[at].replacen("[]", "", 1).trim_end().to_string();
            insert(&mut out, at + 1, written(entry, UNDER_THE_LIST));
            changed(&out)
        }
        Shape::Block(at) => {
            let held = items(&lines, at);
            let indent = held
                .first()
                .map(|item| " ".repeat(indent_of(lines[item.start])))
                .unwrap_or_else(|| UNDER_THE_LIST.to_string());

            let Some(standing) = held.iter().find(|item| item.watch.path == entry.path) else {
                let after = held.last().map(|item| item.end).unwrap_or(at + 1);
                insert(&mut out, after, written(entry, &indent));
                return changed(&out);
            };
            if &standing.watch == entry {
                return Edit::AlreadySo;
            }
            out.splice(standing.start..standing.end, written(entry, &indent));
            changed(&out)
        }
    }
}

pub fn stop_listed(text: &str, path: &str) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let at = match shape(&lines) {
        Shape::NotOurs(why) => return Edit::NotOurs(why),
        Shape::Missing | Shape::Empty(_) => return Edit::AlreadySo,
        Shape::Block(at) => at,
    };

    let held = items(&lines, at);
    let Some(doomed) = held.iter().find(|item| item.watch.path == path) else {
        return Edit::AlreadySo;
    };

    let mut out: Vec<String> = lines
        .iter()
        .enumerate()
        .filter(|(line, _)| !doomed.holds(*line))
        .map(|(_, line)| (*line).to_string())
        .collect();
    if held.len() == 1 {
        out[at] = out[at].replacen(LIST, "files: []", 1);
    }
    changed(&out)
}

enum Shape {
    Missing,
    Empty(usize),
    Block(usize),
    NotOurs(String),
}

fn shape(lines: &[&str]) -> Shape {
    let Some(at) = lines.iter().position(|line| line.starts_with(LIST)) else {
        return Shape::Missing;
    };
    let rest = lines[at][LIST.len()..].trim();
    let value = match rest.find(" #") {
        Some(comment) => rest[..comment].trim(),
        None if rest.starts_with('#') => "",
        None => rest,
    };
    match value {
        "" => Shape::Block(at),
        "[]" => Shape::Empty(at),
        _ => Shape::NotOurs(format!(
            "the {LIST} list in this file is not written one entry to a line, and this console \
             edits no shape it did not write. Add the path by hand"
        )),
    }
}

fn items(lines: &[&str], list: usize) -> Vec<Item> {
    let mut items: Vec<Item> = Vec::new();
    let mut dash: Option<usize> = None;

    for (at, line) in lines.iter().enumerate().skip(list + 1) {
        let held = line.trim();
        if held.is_empty() || held.starts_with('#') {
            continue;
        }
        let indent = indent_of(line);
        let starts = held.starts_with('-');
        match dash {
            None if starts => dash = Some(indent),
            None => break,
            Some(column) if indent == column && starts => {}
            Some(column) if indent > column => {
                if let Some(item) = items.last_mut() {
                    item.end = at + 1;
                    if let Some(size) = size_of(held) {
                        item.watch.ceiling_bytes = Some(size);
                    }
                }
                continue;
            }
            Some(_) => break,
        }
        items.push(Item {
            start: at,
            end: at + 1,
            watch: opened(held[1..].trim()),
        });
    }
    items
}

fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn size_of(field: &str) -> Option<u64> {
    let written = unquoted(field.strip_prefix(SIZE)?.trim()).to_ascii_lowercase();
    let digits = written
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(written.len());
    let count: u64 = written[..digits].parse().ok()?;
    let unit = written[digits..].trim();
    let by = UNITS.iter().find(|(name, _)| *name == unit)?.1;
    count.checked_mul(by)
}

fn shown(bytes: u64) -> String {
    for (name, by) in UNITS {
        if *by > 1 && bytes >= *by && bytes.is_multiple_of(*by) {
            return format!("{}{name}", bytes / by);
        }
    }
    bytes.to_string()
}

fn written(entry: &Watch, indent: &str) -> Vec<String> {
    match entry.ceiling_bytes {
        None => vec![format!("{indent}- {}", quoted(&entry.path))],
        Some(bytes) => vec![
            format!("{indent}- path: {}", quoted(&entry.path)),
            format!("{indent}  {SIZE} {}", shown(bytes)),
        ],
    }
}

fn insert(out: &mut Vec<String>, at: usize, lines: Vec<String>) {
    for (moved, line) in lines.into_iter().enumerate() {
        out.insert(at + moved, line);
    }
}

fn changed(out: &[String]) -> Edit {
    Edit::Changed {
        text: joined(out),
        entries: 1,
    }
}
