const COLLECTORS: &str = "collectors:";

const SCHEDULE: &str = "schedule:";

const ITEM: &str = "  - ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    Changed(String),
    AlreadySo,
    NotOurs(String),
}

pub fn watching(text: &str, name: &str) -> Option<bool> {
    match block(text, COLLECTORS)? {
        Block::Missing => None,
        Block::Listed(items) => Some(items.iter().any(|listed| listed == name)),
    }
}

pub fn add(text: &str, name: &str, every_seconds: u32) -> Edit {
    let listed = match block(text, COLLECTORS) {
        None => return Edit::NotOurs(shape_we_do_not_edit(COLLECTORS)),
        Some(Block::Missing) => {
            return Edit::AlreadySo;
        }
        Some(Block::Listed(items)) => items,
    };

    if listed.iter().any(|item| item == name) {
        return match scheduled(text, name) {
            Some(true) => Edit::AlreadySo,
            Some(false) => with_period(text, name, every_seconds),
            None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
        };
    }

    let with_the_collector = insert(text, COLLECTORS, &format!("{ITEM}{name}"));
    match scheduled(&with_the_collector, name) {
        Some(true) => Edit::Changed(with_the_collector),
        Some(false) => match with_period(&with_the_collector, name, every_seconds) {
            Edit::Changed(text) => Edit::Changed(text),
            Edit::AlreadySo => Edit::Changed(with_the_collector),
            refusal => refusal,
        },
        None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
    }
}

pub fn remove(text: &str, name: &str, every: &[(&str, u32)]) -> Edit {
    let listed = match block(text, COLLECTORS) {
        None => return Edit::NotOurs(shape_we_do_not_edit(COLLECTORS)),
        Some(Block::Missing) => {
            let named: Vec<String> = every
                .iter()
                .filter(|(known, _)| *known != name)
                .map(|(known, _)| format!("{ITEM}{known}"))
                .collect();
            let mut text = text.trim_end().to_string();
            text.push_str(&format!("\n\n{COLLECTORS}\n{}\n", named.join("\n")));
            return Edit::Changed(without_period(&text, name));
        }
        Some(Block::Listed(items)) => items,
    };

    if !listed.iter().any(|item| item == name) {
        return match scheduled(text, name) {
            Some(true) => Edit::Changed(without_period(text, name)),
            _ => Edit::AlreadySo,
        };
    }

    let text = drop_line(text, &format!("{ITEM}{name}"));
    Edit::Changed(without_period(&text, name))
}

fn with_period(text: &str, name: &str, every_seconds: u32) -> Edit {
    match block(text, SCHEDULE) {
        None => Edit::NotOurs(shape_we_do_not_edit(SCHEDULE)),
        Some(Block::Missing) => {
            let mut text = text.trim_end().to_string();
            text.push_str(&format!("\n\n{SCHEDULE}\n  {name}: {every_seconds}\n"));
            Edit::Changed(text)
        }
        Some(Block::Listed(_)) => Edit::Changed(insert(
            text,
            SCHEDULE,
            &format!("  {name}: {every_seconds}"),
        )),
    }
}

fn without_period(text: &str, name: &str) -> String {
    let prefix = format!("  {name}:");
    text.lines()
        .filter(|line| !line.starts_with(&prefix))
        .map(|line| format!("{line}\n"))
        .collect()
}

fn scheduled(text: &str, name: &str) -> Option<bool> {
    match block(text, SCHEDULE)? {
        Block::Missing => Some(false),
        Block::Listed(_) => Some(
            text.lines()
                .any(|line| line.starts_with(&format!("  {name}:"))),
        ),
    }
}

enum Block {
    Missing,
    Listed(Vec<String>),
}

fn block(text: &str, heading: &str) -> Option<Block> {
    let mut lines = text.lines();
    let Some(start) = lines.position(|line| line.trim_end() == heading) else {
        return match text.lines().any(|line| line.starts_with(heading)) {
            true => None,
            false => Some(Block::Missing),
        };
    };

    let mut items = Vec::new();
    for line in text.lines().skip(start + 1) {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if !line.starts_with(' ') {
            break;
        }
        let held = line.trim_start().trim_start_matches("- ").trim();
        items.push(held.split(':').next().unwrap_or(held).trim().to_string());
    }
    Some(Block::Listed(items))
}

fn insert(text: &str, heading: &str, line: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    let mut placed = false;

    for source in text.lines() {
        if inside && !placed && (source.trim().is_empty() || !source.starts_with(' ')) {
            out.push_str(line);
            out.push('\n');
            placed = true;
            inside = false;
        }
        out.push_str(source);
        out.push('\n');
        if source.trim_end() == heading {
            inside = true;
        }
    }
    if inside && !placed {
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn drop_line(text: &str, wanted: &str) -> String {
    text.lines()
        .filter(|line| line.trim_end() != wanted)
        .map(|line| format!("{line}\n"))
        .collect()
}

fn shape_we_do_not_edit(heading: &str) -> String {
    format!(
        "the {heading} in this file is not written as a block of lines, and this command edits \
         no shape it did not write. Add the line by hand, or write the file again with `vigild \
         configure --force`"
    )
}
