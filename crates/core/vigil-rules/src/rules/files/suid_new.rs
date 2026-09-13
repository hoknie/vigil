use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::file_finding::{FileFinding, build, standing};
use super::file_view::{Family, FileView};
use crate::{Rule, RuleContext};

pub struct FileSuidNew;

impl Rule for FileSuidNew {
    fn name(&self) -> &'static str {
        "file_suid_new"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = FileView::new(key, before);
        let now = FileView::new(key, after);
        if !now.is(Family::File) || !now.present() {
            return None;
        }
        if was.runs_as_its_owner() || !now.runs_as_its_owner() {
            return None;
        }

        Some(build(
            FileFinding {
                kind: KnownKind::FileSuidNew,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "file",
                title: format!(
                    "{} now runs as the account that owns it: mode {}, and it was {}",
                    now.path(),
                    now.mode().unwrap_or("unknown"),
                    was.mode().unwrap_or("unknown")
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    standing(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "whoever may run this program now runs it as its owner, and if that owner is root, so does everybody".into(),
                    },
                ],
            },
            ctx,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        FileSuidNew.apply(change, &mut ctx)
    }

    fn mode_went(before: &str, after: &str) -> Change {
        Change::Changed {
            key: "file|/usr/bin/at".into(),
            before: fixture::watched_file("/usr/bin/at", before, "a9"),
            after: fixture::watched_file("/usr/bin/at", after, "a9"),
        }
    }

    #[test]
    fn a_watched_file_that_became_setuid_is_reported_and_named() {
        let finding = apply(&mode_went("0755", "4755")).expect("fires");

        assert_eq!(finding.finding_key, "file|/usr/bin/at");
        assert_eq!(finding.kind.as_str(), "file.suid.new");
        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn a_watched_file_that_became_setgid_is_the_same_finding() {
        assert!(apply(&mode_went("0755", "2755")).is_some());
    }

    #[test]
    fn a_file_that_was_setuid_all_along_is_not_a_new_one() {
        assert!(apply(&mode_went("4755", "4700")).is_none());
    }

    #[test]
    fn a_bit_that_was_taken_off_is_not_a_finding_this_vocabulary_has_a_kind_for() {
        assert!(apply(&mode_went("4755", "0755")).is_none());
    }

    #[test]
    fn a_first_reading_of_a_host_whose_setuid_programs_are_its_own_says_nothing() {
        let change = Change::Added {
            key: "file|/usr/bin/at".into(),
            after: fixture::watched_file("/usr/bin/at", "4755", "a9"),
        };

        assert!(
            apply(&change).is_none(),
            "every host ships with setuid programs, and the first reading of one of them is \
             the baseline this product is quiet because of"
        );
    }
}
