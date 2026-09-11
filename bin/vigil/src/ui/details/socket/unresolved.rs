use ratatui::text::Line;

use super::lines::named;
use crate::ui::helpers::layout::wrap;
use crate::ui::{Look, Report};

pub(super) fn unresolved(report: &mut Report, count: usize, look: Look, width: usize) {
    report.push(Line::styled("   OWNER NOT RESOLVED", look.palette.alarm()));
    report.blank();
    named(report, look, "sockets", &count.to_string(), width);
    report.blank();
    for line in wrap::wrap(
        "These are not a program called \"unknown\", and not sockets with no owner. Resolving \
         one means reading /proc/<pid>/fd for every process on the host; that was refused. \
         The summary screen says which privilege is missing.",
        width.saturating_sub(5),
    ) {
        report.push(Line::styled(format!("   {line}"), look.palette.alarm()));
    }
}
