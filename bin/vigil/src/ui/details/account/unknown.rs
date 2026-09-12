use super::lines::{headline, say};
use crate::ui::screens::accounts::Row;
use crate::ui::{Look, Report};

pub(super) fn unknown(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    headline(report, look, &row.key, false);
    say(
        report,
        look.palette.quiet(),
        "This console has no screen for this kind of object. Both halves ship in one package, \
         so this is a screen nobody has written yet, not an agent that ran ahead. The key above \
         is what a suppression matches.",
        width,
    );
    report.blank();
}
