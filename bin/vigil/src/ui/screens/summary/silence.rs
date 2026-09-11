use ratatui::text::Line;
use vigil_model::AgentStatus;

use super::lines::bullet;
use crate::ui::helpers::layout::section;
use crate::ui::{Look, Report};

pub(super) fn silence(report: &mut Report, agent: &AgentStatus, look: Look, width: usize) {
    if agent.silence.suppressions.is_empty() {
        return;
    }

    report.push(section::rule(look, "WHAT IS NOT BEING SAID", width));
    report.push(Line::raw(format!(
        "   {} suppression(s) from the configuration, {} finding(s) silenced:",
        agent.silence.suppressions.len(),
        agent.silence.suppressed
    )));
    for suppression in &agent.silence.suppressions {
        for line in bullet(suppression, width) {
            report.push(line);
        }
    }
    report.blank();
}
