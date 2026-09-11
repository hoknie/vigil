use vigil_model::AgentStatus;

use super::lines::bullet;
use crate::ui::helpers::layout::section;
use crate::ui::{Look, Report};

pub(super) fn limitations(report: &mut Report, agent: &AgentStatus, look: Look, width: usize) {
    if agent.limitations.is_empty() {
        return;
    }

    report.push(section::rule(look, "NOT YET", width));
    for limitation in &agent.limitations {
        for line in bullet(limitation, width) {
            report.push(line);
        }
    }
}
