use ratatui::text::Line;
use vigil_model::{AgentStatus, BufferStatus, ReporterStatus};

use super::collectors::{MARK, NOT_REPORTED};
use super::lines::note;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::words::{moment, size};
use crate::ui::{Look, Report};

pub(super) fn reporters(report: &mut Report, agent: &AgentStatus, look: Look, width: usize) {
    report.push(section::rule(look, "WHERE FINDINGS GO", width));

    if agent.reporters.is_empty() {
        report.push(Line::raw("   none configured · findings stay on this host"));
        report.blank();
        return;
    }

    report.push(Line::styled(
        format!(
            "   {}",
            column::columns(&[
                ("", 1),
                ("NAME", 14),
                ("DELIVERED", 10),
                ("FAILED", 7),
                ("LAST", 10),
                ("WAITING", 12),
            ])
        ),
        look.palette.quiet(),
    ));
    for reporter in &agent.reporters {
        let held = buffer(agent, &reporter.name);
        let losing = held.is_some_and(|buffer| buffer.dropped_total > 0);
        report.push(Line::raw(format!(
            "   {}",
            column::columns(&[
                (
                    match losing || reporter.last_error.is_some() {
                        true => MARK,
                        false => "",
                    },
                    1
                ),
                (&reporter.name, 14),
                (&reporter.deliveries.to_string(), 10),
                (&reporter.failures.to_string(), 7),
                (
                    reporter
                        .last_sent_at
                        .as_deref()
                        .map(moment::time_of_day)
                        .unwrap_or("never"),
                    10
                ),
                (&waiting(agent, held), 12),
            ])
        )));
        for sentence in says(reporter, held) {
            for line in note(&sentence, width) {
                report.push(line);
            }
        }
    }
    report.blank();
}

fn buffer<'a>(agent: &'a AgentStatus, receiver: &str) -> Option<&'a BufferStatus> {
    agent
        .buffers
        .as_ref()?
        .iter()
        .find(|buffer| buffer.receiver == receiver)
}

fn waiting(agent: &AgentStatus, held: Option<&BufferStatus>) -> String {
    match (agent.buffers.is_some(), held) {
        (false, _) => NOT_REPORTED.to_string(),
        (true, None) => NOTHING_HELD.to_string(),
        (true, Some(buffer)) => format!("{} of {}", buffer.pending, buffer.pending_ceiling),
    }
}

fn says(reporter: &ReporterStatus, held: Option<&BufferStatus>) -> Vec<String> {
    let mut said = Vec::new();

    match held {
        Some(buffer) if buffer.dropped_total > 0 => said.push(format!(
            "{} finding(s) were dropped: this buffer was full at {} finding(s) or {}, and the \
             oldest went to make room. What was dropped was never sent and is on no screen. {}",
            buffer.dropped_total,
            buffer.pending_ceiling,
            size::bytes(buffer.bytes_ceiling),
            holding(buffer)
        )),
        Some(buffer) if buffer.pending > 0 => said.push(format!(
            "{} A receiver that is behind is caught up on the next delivery, and nothing is \
             lost until the ceiling is reached.",
            holding(buffer)
        )),
        _ => {}
    }
    if let Some(error) = &reporter.last_error {
        said.push(format!("last failure: {error}"));
    }
    said
}

fn holding(buffer: &BufferStatus) -> String {
    format!(
        "{} of {} finding(s) are waiting to go, {} of {}{}.",
        buffer.pending,
        buffer.pending_ceiling,
        size::bytes(buffer.bytes),
        size::bytes(buffer.bytes_ceiling),
        match &buffer.oldest_at {
            Some(oldest) => format!(", the oldest found at {}", moment::time_of_day(oldest)),
            None => String::new(),
        }
    )
}

const NOTHING_HELD: &str = "\u{2014}";
