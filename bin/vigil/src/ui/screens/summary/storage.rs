use ratatui::text::Line;
use vigil_model::{AgentStatus, StoreStatus};

use crate::ui::helpers::layout::field;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Look, Report};

const NAME_WIDTH: usize = 16;

pub(super) fn storage(report: &mut Report, agent: &AgentStatus, look: Look, width: usize) {
    report.push(section::rule(look, "WHAT IS KEPT ON DISK", width));

    match &agent.store {
        None => said(
            report,
            look,
            "This agent does not report what its local history holds. The findings are on disk \
             under its state directory, and `jq` reads them.",
            width,
        ),
        Some(store) if store.is_empty() => {
            said(
                report,
                look,
                "Nothing recorded yet: this store holds no finding at all.",
                width,
            );
            if let Some(path) = &store.journal_path {
                named(report, look, "journal", path, width);
            }
        }
        Some(store) => numbers(report, store, look, width),
    }

    named(
        report,
        look,
        "waiting to send",
        "nothing is buffered: this agent sends as it finds",
        width,
    );
    report.blank();
}

fn numbers(report: &mut Report, store: &StoreStatus, look: Look, width: usize) {
    named(
        report,
        look,
        "findings kept",
        &format!(
            "{} of {}{}",
            store.records.held,
            store.records.ceiling,
            match &store.oldest_at {
                Some(oldest) => format!("      oldest  {oldest}"),
                None => String::new(),
            }
        ),
        width,
    );
    named(
        report,
        look,
        "journal",
        &format!(
            "{} of {}{}",
            megabytes(store.bytes.held),
            megabytes(store.bytes.ceiling),
            match &store.journal_path {
                Some(path) => format!("      {path}"),
                None => String::new(),
            }
        ),
        width,
    );
    named(
        report,
        look,
        "dropped",
        &format!(
            "{} past the retention window · {} at the ceiling",
            store.dropped.past_the_window, store.dropped.at_the_ceiling
        ),
        width,
    );
    if store.damaged > 0 {
        named(
            report,
            look,
            "unreadable",
            &format!(
                "{} line(s) were skipped when the agent started, and counted",
                store.damaged
            ),
            width,
        );
    }
}

fn megabytes(bytes: u64) -> String {
    match bytes < 1024 * 1024 {
        true => format!("{} kB", bytes / 1024),
        false => format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)),
    }
}

fn named(report: &mut Report, look: Look, name: &str, value: &str, width: usize) {
    for line in field::lines(look, name, value, NAME_WIDTH, width) {
        report.push(line);
    }
}

fn said(report: &mut Report, look: Look, sentence: &str, width: usize) {
    for line in wrap::wrap(sentence, width.saturating_sub(5)) {
        report.push(Line::styled(format!("   {line}"), look.palette.quiet()));
    }
}
