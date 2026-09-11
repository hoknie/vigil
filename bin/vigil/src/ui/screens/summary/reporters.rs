use ratatui::text::Line;
use vigil_model::AgentStatus;

use super::lines::note;
use crate::ui::helpers::layout::column;
use crate::ui::helpers::layout::section;
use crate::ui::helpers::words::moment;
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
            column::columns(&[("NAME", 14), ("DELIVERED", 10), ("FAILED", 7), ("LAST", 10)])
        ),
        look.palette.quiet(),
    ));
    for reporter in &agent.reporters {
        report.push(Line::raw(format!(
            "   {}",
            column::columns(&[
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
            ])
        )));
        if let Some(error) = &reporter.last_error {
            for line in note(&format!("last failure: {error}"), width) {
                report.push(line);
            }
        }
    }
    report.blank();
}
