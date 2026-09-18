use crate::types::switched::Switched;

const ENABLED: &str = "enabled:";

const INDENT: usize = 2;

pub fn switched(text: &str, name: &str, enabled: bool) -> Switched {
    let lines: Vec<&str> = text.lines().collect();
    let Some(at) = lines.iter().position(|line| is_the_key(line, name)) else {
        return Switched::NotOurs(format!(
            "no line of this file begins with `{name}:`, and this command edits no shape it did \
             not write. Write `enabled: {enabled}` under the block by hand"
        ));
    };
    let after_the_key = lines[at][name.len() + 1..].trim();
    if !after_the_key.is_empty() && !after_the_key.starts_with('#') {
        return Switched::NotOurs(format!(
            "`{name}:` is written on one line, and this command edits no shape it did not \
             write. Write `enabled: {enabled}` into the block by hand"
        ));
    }

    let (end, indent) = body(&lines, at);
    let found = (at + 1..end).find(|index| {
        let line = lines[*index];
        width(line) == indent && line.trim_start().starts_with(ENABLED)
    });

    let mut edited: Vec<String> = lines.iter().map(|line| line.to_string()).collect();
    match found {
        None if enabled => return Switched::AlreadySo,
        None => edited.insert(at + 1, format!("{}{ENABLED} {enabled}", " ".repeat(indent))),
        Some(index) => match rewritten(lines[index], indent, enabled) {
            Err(why) => return Switched::NotOurs(why),
            Ok(None) => return Switched::AlreadySo,
            Ok(Some(line)) => edited[index] = line,
        },
    }

    let mut out = edited.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    Switched::Changed(out)
}

pub fn new_block(name: &str, every_seconds: u32) -> String {
    format!("{name}:\n{}schedule: {every_seconds}\n", " ".repeat(INDENT))
}

pub fn with_block(text: &str, block: &str) -> String {
    let kept = text.trim_end();
    match kept.is_empty() {
        true => block.to_string(),
        false => format!("{kept}\n{block}"),
    }
}

fn is_the_key(line: &str, name: &str) -> bool {
    line.strip_prefix(name)
        .is_some_and(|rest| rest.starts_with(':'))
}

fn body(lines: &[&str], at: usize) -> (usize, usize) {
    let mut end = at + 1;
    let mut indent = None;
    while end < lines.len() {
        let line = lines[end];
        let held = line.trim_start();
        if held.is_empty() || held.starts_with('#') {
            end += 1;
            continue;
        }
        if width(line) == 0 {
            break;
        }
        indent.get_or_insert(width(line));
        end += 1;
    }
    (end, indent.unwrap_or(INDENT))
}

fn width(line: &str) -> usize {
    line.len() - line.trim_start_matches(' ').len()
}

fn rewritten(line: &str, indent: usize, enabled: bool) -> Result<Option<String>, String> {
    let rest = &line[indent + ENABLED.len()..];
    let spaces = rest.len() - rest.trim_start().len();
    let value_and_tail = &rest[spaces..];
    let value_ends = value_and_tail
        .find(char::is_whitespace)
        .unwrap_or(value_and_tail.len());
    let (value, tail) = value_and_tail.split_at(value_ends);

    let now = match value {
        "true" | "True" | "TRUE" => true,
        "false" | "False" | "FALSE" => false,
        _ => {
            return Err(format!(
                "`{}` is not true or false, and this command does not guess what it meant. \
                 Write `enabled: {enabled}` by hand",
                line.trim()
            ));
        }
    };
    if now == enabled {
        return Ok(None);
    }
    Ok(Some(format!(
        "{}{ENABLED}{}{enabled}{tail}",
        " ".repeat(indent),
        " ".repeat(spaces.max(1))
    )))
}
