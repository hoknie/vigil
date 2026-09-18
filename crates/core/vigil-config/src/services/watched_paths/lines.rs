use crate::helpers::quoting::unquoted;
use crate::types::watch::Watch;

pub(super) struct Item {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) watch: Watch,
}

impl Item {
    pub(super) fn holds(&self, line: usize) -> bool {
        line >= self.start && line < self.end
    }
}

pub(super) fn opened(first: &str) -> Watch {
    match first.strip_prefix("path:") {
        Some(value) => Watch::of(unquoted(value.trim()), None),
        None => match first.split_once(':') {
            Some(_) => Watch::of(String::new(), None),
            None => Watch::of(unquoted(first), None),
        },
    }
}

pub(super) fn joined(lines: &[String]) -> String {
    let mut out = lines.join("\n");
    out.push('\n');
    out
}
