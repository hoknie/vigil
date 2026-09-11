use std::collections::BTreeMap;

use super::event::Event;
use super::reading::AuditReading;
use super::record::Record;
use super::text::unquote;

pub const AUDIT_KEY: &str = "vigil_exec";

pub(super) const TRAILING_LINES: usize = 64;

const READ_RECORD_TYPES: [&str; 4] = ["SYSCALL", "EXECVE", "CWD", "PATH"];

pub fn record_is_read(line: &[u8]) -> bool {
    let text = String::from_utf8_lossy(line);
    text.split_whitespace()
        .find_map(|token| token.strip_prefix("type="))
        .is_some_and(|kind| READ_RECORD_TYPES.contains(&unquote(kind)))
}

pub fn parse_audit_log(chunk: &[u8], with_arguments: bool) -> AuditReading {
    let mut events: Vec<Event> = Vec::new();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    let mut offset = 0usize;
    let mut line_number = 0usize;

    for line in chunk.split_inclusive(|byte| *byte == b'\n') {
        let started_at = offset;
        offset += line.len();
        if !line.ends_with(b"\n") {
            offset = started_at;
            break;
        }
        line_number += 1;

        let text = String::from_utf8_lossy(line);
        let Some(record) = Record::parse(text.trim_end()) else {
            continue;
        };

        let slot = *index.entry(record.event.clone()).or_insert_with(|| {
            events.push(Event::new(record.event.clone(), started_at));
            events.len() - 1
        });
        events[slot].last_line = line_number;
        events[slot].absorb(record);
    }

    let mut executions = Vec::new();
    let mut unnamed = 0usize;
    let mut consumed = offset;

    for event in &events {
        if !event.complete() {
            if event.half_written() && event.last_line + TRAILING_LINES > line_number {
                consumed = consumed.min(event.started_at);
            }
            continue;
        }
        if !event.ours() || !event.succeeded {
            continue;
        }
        match event.execution(with_arguments) {
            Some(execution) => executions.push(execution),
            None => unnamed += 1,
        }
    }

    AuditReading {
        executions,
        unnamed,
        consumed,
    }
}
