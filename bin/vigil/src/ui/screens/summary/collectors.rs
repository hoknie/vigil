use ratatui::text::Line;
use vigil_model::{AgentStatus, CollectorState, CollectorStatus};

use super::lines::note;
use super::room::{ROOM_FOR_THE_NEXT_READING, ROOM_FOR_TWO_COLUMNS};
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::words::moment;
use crate::ui::{Look, Report};

pub(super) const NOT_REPORTED: &str = "not reported";

const NOTHING: &str = "—";

pub(super) fn collectors(
    report: &mut Report,
    agent: &AgentStatus,
    look: Look,
    width: usize,
    saying: bool,
) {
    report.push(section::rule(look, "COLLECTORS", width));

    let wide = width >= ROOM_FOR_TWO_COLUMNS as usize;
    let roomy = width >= ROOM_FOR_THE_NEXT_READING as usize;
    let mut headings = vec![
        ("", 1),
        ("NAME", 10),
        ("STATE", 11),
        ("EVERY", 12),
        ("LAST READ", 10),
        ("ITEMS", 6),
        ("READINGS", 7),
        ("FAILURES", 7),
    ];
    if roomy {
        headings.push(("NEXT", 10));
    }
    if wide {
        headings.push(("TOOK", 8));
        headings.push(("BASELINE", 8));
    }
    report.push(Line::styled(
        format!("   {}", column::columns(&headings)),
        look.palette.quiet(),
    ));

    for collector in &agent.collectors {
        let cells = row(collector);
        let said = says(collector);
        let mut drawn: Vec<(&str, usize)> = vec![
            (
                match said.is_empty() {
                    true => "",
                    false => MARK,
                },
                1,
            ),
            (&collector.name, 10),
            (collector.state.as_str(), 11),
            (&cells.every, 12),
            (&cells.last_read, 10),
            (&cells.items, 6),
            (&cells.readings, 7),
            (&cells.failures, 7),
        ];
        if roomy {
            drawn.push((&cells.next, 10));
        }
        if wide {
            drawn.push((&cells.took, 8));
            drawn.push((cells.baseline, 8));
        }

        report.push(
            Line::raw(format!("   {}", column::columns(&drawn)))
                .style(look.palette.collector(&collector.state)),
        );

        if !saying {
            continue;
        }
        for sentence in said {
            for line in note(&sentence, width) {
                report.push(line);
            }
        }
    }

    if !saying && agent.collectors.iter().any(|it| !says(it).is_empty()) {
        report.push(Line::styled(
            format!("   {MARK} a collector with something to say about itself: press d to read it"),
            look.palette.quiet(),
        ));
    }
    report.blank();
}

pub(super) const MARK: &str = "!";

pub(super) fn says(collector: &CollectorStatus) -> Vec<String> {
    let mut said = Vec::new();
    if let Some(reason) = collector.reason.as_deref()
        && !reason.is_empty()
    {
        said.push(reason.to_string());
    }
    if collector.skipped > 0 {
        said.push(skipped(collector));
    }
    if let Some(error) = &collector.last_error {
        said.push(format!("last failure: {error}"));
    }
    said
}

fn skipped(collector: &CollectorStatus) -> String {
    format!(
        "{} reading(s) dropped: the slot came round while the previous one was still \
         running, and two readings back to back compare a host that did not move",
        collector.skipped
    )
}

struct Cells {
    every: String,
    next: String,
    last_read: String,
    items: String,
    readings: String,
    failures: String,
    took: String,
    baseline: &'static str,
}

fn row(collector: &CollectorStatus) -> Cells {
    let off = collector.state == CollectorState::Off;
    let nothing = NOTHING.to_string();

    Cells {
        every: match (off, collector.every_seconds) {
            (true, _) => nothing.clone(),
            (false, Some(seconds)) => format!("every {seconds} s"),
            (false, None) => NOT_REPORTED.to_string(),
        },
        next: match (off, collector.next_run_at.as_deref()) {
            (true, _) => nothing.clone(),
            (false, Some(when)) => moment::time_of_day(when).to_string(),
            (false, None) => "not yet".to_string(),
        },
        last_read: match off {
            true => nothing.clone(),
            false => collector
                .last_run_at
                .as_deref()
                .map(moment::time_of_day)
                .unwrap_or("never")
                .to_string(),
        },
        items: match off {
            true => nothing.clone(),
            false => collector.items.to_string(),
        },
        readings: match off {
            true => nothing.clone(),
            false => collector.readings.to_string(),
        },
        failures: match off {
            true => nothing.clone(),
            false => collector.failures.to_string(),
        },
        took: match (off, collector.duration_ms) {
            (false, Some(milliseconds)) => format!("{milliseconds} ms"),
            _ => nothing,
        },
        baseline: match (off, collector.baseline) {
            (true, _) => NOTHING,
            (false, true) => "yes",
            (false, false) => "not yet",
        },
    }
}
