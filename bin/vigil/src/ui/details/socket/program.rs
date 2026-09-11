use ratatui::text::Line;

use super::lines::named;
use crate::ui::helpers::layout::wrap;
use crate::ui::screens::ports;
use crate::ui::{Look, Report};

pub(super) fn program(report: &mut Report, path: &str, count: usize, look: Look, width: usize) {
    report.push(Line::styled(
        format!("   {}", ports::basename(path)),
        look.palette.heading(),
    ));
    report.blank();
    named(report, look, "program", path, width);
    named(report, look, "sockets", &count.to_string(), width);
    report.blank();
    for line in wrap::wrap(
        "The sockets are listed under this heading. Press Esc for the list, then move to one \
         of them for its detail.",
        width.saturating_sub(5),
    ) {
        report.push(Line::styled(format!("   {line}"), look.palette.quiet()));
    }
}
