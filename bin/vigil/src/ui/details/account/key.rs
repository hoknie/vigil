use super::lines::{headline, named, say};
use crate::ui::helpers::layout::section;
use crate::ui::screens::accounts;
use crate::ui::screens::accounts::Row;
use crate::ui::{Look, Report};

pub(super) fn key(report: &mut Report, row: &Row<'_>, look: Look, width: usize) {
    let item = row.item;
    let user = accounts::text(item, "user").unwrap_or("?");
    headline(report, look, user, true);

    if !accounts::readable(item) {
        say(
            report,
            look.palette.alarm(),
            &format!(
                "{} was not readable. That is a refusal, not an account with no keys: there \
                 may be keys here that were never read.",
                accounts::text(item, "source").unwrap_or("This file")
            ),
            width,
        );
        report.blank();
        return;
    }

    named(
        report,
        look,
        "algorithm",
        accounts::text(item, "algorithm").unwrap_or("?"),
        width,
    );
    named(
        report,
        look,
        "comment",
        accounts::text(item, "comment").unwrap_or("(none)"),
        width,
    );
    named(
        report,
        look,
        "restricted",
        match accounts::text(item, "options") {
            Some(options) => options,
            None => "no command, source or expiry restriction on this key",
        },
        width,
    );
    named(
        report,
        look,
        "file",
        accounts::text(item, "source").unwrap_or("?"),
        width,
    );
    report.blank();

    report.push(section::rule(look, "FINGERPRINT", width));
    named(
        report,
        look,
        "",
        accounts::text(item, "fingerprint").unwrap_or("no fingerprint"),
        width,
    );
    say(
        report,
        look.palette.quiet(),
        "Compare it with `ssh-keygen -lf` on the key you believe should be there.",
        width,
    );
    report.blank();
}
