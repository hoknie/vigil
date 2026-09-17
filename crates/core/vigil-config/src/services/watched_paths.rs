use crate::helpers::quoting::unquoted;
use crate::services::suppressions::Edit;
use crate::types::watch::Watch;

const MODULE: &str = "files:";

const LIST: &str = "paths:";

const EMPTY: &str = "paths: []";

const UNDER_THE_MODULE: &str = "  ";

const UNDER_THE_LIST: &str = "    ";

pub fn paths(text: &str) -> Vec<Watch> {
    let lines: Vec<&str> = text.lines().collect();
    let Shape::Block(at) = shape(&lines) else {
        return Vec::new();
    };

    items(&lines, at)
        .into_iter()
        .map(|item| item.watch)
        .collect()
}

pub fn put(text: &str, entry: &Watch) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<String> = lines.iter().map(|line| (*line).to_string()).collect();

    match shape(&lines) {
        Shape::NotOurs(why) => Edit::NotOurs(why),
        Shape::Missing => {
            while out.last().is_some_and(|line| line.trim().is_empty()) {
                out.pop();
            }
            out.push(String::new());
            out.push(MODULE.to_string());
            out.push(format!("{UNDER_THE_MODULE}{LIST}"));
            out.extend(entry.lines(UNDER_THE_LIST));
            Edit::Changed {
                text: joined(&out),
                entries: 1,
            }
        }
        Shape::NoList(at) => {
            out.insert(at + 1, format!("{UNDER_THE_MODULE}{LIST}"));
            for (moved, line) in entry.lines(UNDER_THE_LIST).into_iter().enumerate() {
                out.insert(at + 2 + moved, line);
            }
            Edit::Changed {
                text: joined(&out),
                entries: 1,
            }
        }
        Shape::Empty(at) => {
            out[at] = out[at].replace(EMPTY, LIST);
            for (moved, line) in entry.lines(UNDER_THE_LIST).into_iter().enumerate() {
                out.insert(at + 1 + moved, line);
            }
            Edit::Changed {
                text: joined(&out),
                entries: 1,
            }
        }
        Shape::Block(at) => written(out, &lines, at, entry),
    }
}

pub fn stop(text: &str, path: &str) -> Edit {
    let lines: Vec<&str> = text.lines().collect();
    let at = match shape(&lines) {
        Shape::NotOurs(why) => return Edit::NotOurs(why),
        Shape::Missing | Shape::NoList(_) | Shape::Empty(_) => return Edit::AlreadySo,
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
        out[at] = out[at].replace(LIST, EMPTY);
    }

    Edit::Changed {
        text: joined(&out),
        entries: 1,
    }
}

enum Shape {
    Missing,
    NoList(usize),
    Empty(usize),
    Block(usize),
    NotOurs(String),
}

struct Item {
    start: usize,
    end: usize,
    watch: Watch,
}

impl Item {
    fn holds(&self, line: usize) -> bool {
        line >= self.start && line < self.end
    }
}

fn shape(lines: &[&str]) -> Shape {
    let Some(module) = lines.iter().position(|line| line.starts_with(MODULE)) else {
        return Shape::Missing;
    };
    if !lines[module][MODULE.len()..].trim().is_empty() {
        return Shape::NotOurs(shape_we_do_not_edit());
    }

    for (at, line) in lines.iter().enumerate().skip(module + 1) {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let held = line.trim_start();
        if !held.starts_with(LIST) {
            continue;
        }
        return match held[LIST.len()..].trim() {
            "" => Shape::Block(at),
            "[]" => Shape::Empty(at),
            _ => Shape::NotOurs(shape_we_do_not_edit()),
        };
    }
    Shape::NoList(module)
}

fn items(lines: &[&str], list: usize) -> Vec<Item> {
    let indent = lines[list].len() - lines[list].trim_start().len();
    let mut items: Vec<Item> = Vec::new();

    for (at, line) in lines.iter().enumerate().skip(list + 1) {
        let held = line.trim();
        if held.is_empty() || held.starts_with('#') {
            continue;
        }
        if line.len() - line.trim_start().len() <= indent {
            break;
        }
        match held.strip_prefix('-') {
            Some(first) => items.push(Item {
                start: at,
                end: at + 1,
                watch: opened(first.trim()),
            }),
            None => {
                if let Some(item) = items.last_mut() {
                    item.end = at + 1;
                    if let Some(ceiling) = ceiling_of(held) {
                        item.watch.ceiling_bytes = Some(ceiling);
                    }
                }
            }
        }
    }
    items
}

fn opened(first: &str) -> Watch {
    match first.strip_prefix("path:") {
        Some(value) => Watch::of(unquoted(value.trim()), None),
        None => match first.split_once(':') {
            Some(_) => Watch::of(String::new(), None),
            None => Watch::of(unquoted(first), None),
        },
    }
}

fn ceiling_of(field: &str) -> Option<u64> {
    field
        .strip_prefix("ceiling_bytes:")
        .and_then(|value| unquoted(value.trim()).parse().ok())
}

fn written(mut out: Vec<String>, lines: &[&str], at: usize, entry: &Watch) -> Edit {
    let indent = lines[at].len() - lines[at].trim_start().len();
    let under = " ".repeat(indent + 2);
    let held = items(lines, at);

    let Some(standing) = held.iter().find(|item| item.watch.path == entry.path) else {
        let after = held.last().map(|item| item.end).unwrap_or(at + 1);
        for (moved, line) in entry.lines(&under).into_iter().enumerate() {
            out.insert(after + moved, line);
        }
        return Edit::Changed {
            text: joined(&out),
            entries: 1,
        };
    };

    if &standing.watch == entry {
        return Edit::AlreadySo;
    }

    let mut written: Vec<String> = out.drain(..standing.start).collect();
    written.extend(entry.lines(&under));
    written.extend(out.drain(standing.end - standing.start..));

    Edit::Changed {
        text: joined(&written),
        entries: 1,
    }
}

fn joined(lines: &[String]) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

fn shape_we_do_not_edit() -> String {
    format!(
        "the {MODULE} block in this file is not written as a block of lines, and this console \
         edits no shape it did not write. Add the path by hand, or write the file again with \
         `vigild configure --force`"
    )
}
