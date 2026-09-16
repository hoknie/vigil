use serde_json::Value;

use super::fields::text;
use super::launches::{last_run, program, runs, who};
use crate::helpers::Moment;

const END_OF_THE_NAME: &str = "\0\0";

const NUL_IN_THE_NAME: &str = "\0\u{1}";

const SEEN: &str = "+";

pub(super) fn sorted_on(item: &Value) -> Vec<String> {
    let (name, path) = program(item);
    vec![
        who(item),
        program_key(name, path),
        runs_key(runs(item)),
        last_run_key(last_run(item)),
        first_seen_key(text(item, "first_seen")),
    ]
}

pub(super) fn program_key(name: &str, path: &str) -> String {
    let mut key = String::with_capacity(name.len() + path.len() + END_OF_THE_NAME.len());
    for character in name.chars() {
        match character {
            '\0' => key.push_str(NUL_IN_THE_NAME),
            other => key.push(other),
        }
    }
    key.push_str(END_OF_THE_NAME);
    key.push_str(path);
    key
}

pub(super) fn runs_key(counted: u64) -> String {
    format!("{counted:020}")
}

pub(super) fn last_run_key(last: Option<Moment>) -> String {
    match last {
        Some((seconds, milliseconds, serial)) => {
            format!("{SEEN}{seconds:020}{milliseconds:020}{serial:020}")
        }
        None => String::new(),
    }
}

pub(super) fn first_seen_key(seen: Option<&str>) -> String {
    match seen {
        Some(seen) => format!("{SEEN}{seen}"),
        None => String::new(),
    }
}
