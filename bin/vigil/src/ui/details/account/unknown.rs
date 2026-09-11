use super::lines::{headline, say};
use crate::ui::screens::accounts::Row;
use crate::ui::{Look, Report};

pub(super) fn unknown(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    headline(report, look, &row.key, false);
    say(
        report,
        look.palette.quiet(),
        "This console has no screen for this kind of object: the agent is newer. Both ship in \
         one package and belong installed together. The key above is what a suppression \
         matches.",
        width,
    );
    report.blank();
}
