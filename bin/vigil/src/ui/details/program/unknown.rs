use super::lines::{headline, named, say};
use crate::ui::screens::programs::{Row, text};
use crate::ui::{Look, Report};

pub(super) fn unknown(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    headline(report, look, &row.key, true);

    match text(row.item, "reason") {
        Some(reason) => say(report, look.palette.alarm(), reason, width),
        None => say(
            report,
            look.palette.quiet(),
            "This console has no screen for this kind of object: the agent is newer. Both ship \
             in one package and belong installed together.",
            width,
        ),
    }
    report.blank();
    named(report, look, "object", &row.key, width);
    report.blank();
}
