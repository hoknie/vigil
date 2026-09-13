use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::file_finding::{FileFinding, build, standing};
use super::file_view::{Family, FileView};
use crate::{Rule, RuleContext};

pub struct FileChanged;

impl Rule for FileChanged {
    fn name(&self) -> &'static str {
        "file_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = FileView::new(key, before);
        let now = FileView::new(key, after);
        if !now.is(Family::File) {
            return None;
        }

        let moved = match (was.digest(), now.digest()) {
            (Some(was), Some(now)) => was != now,
            _ => was.present() != now.present(),
        };
        if !moved {
            return None;
        }

        Some(build(
            FileFinding {
                kind: KnownKind::FileChanged,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: "file",
                title: title(&was, &now),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    standing(&now),
                    Evidence {
                        kind: "note".into(),
                        value: format!(
                            "sha256 {} , and at the reading before it was {}",
                            now.digest().unwrap_or("is not known"),
                            was.digest().unwrap_or("not known either")
                        ),
                    },
                ],
            },
            ctx,
        ))
    }
}

fn title(was: &FileView<'_>, now: &FileView<'_>) -> String {
    match (was.present(), now.present()) {
        (true, false) => format!("{} is gone, and this host was watching it", now.path()),
        (false, true) => format!(
            "{} is on this host, and at the reading before it was not",
            now.path()
        ),
        _ => format!(
            "{} holds something other than what it held at the reading before",
            now.path()
        ),
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
        FileChanged.apply(change, &mut ctx)
    }

    fn went(before: serde_json::Value, after: serde_json::Value) -> Change {
        Change::Changed {
            key: "file|/etc/ssh/sshd_config".into(),
            before,
            after,
        }
    }

    #[test]
    fn a_file_whose_content_moved_is_reported_under_the_path_it_is_watched_by() {
        let finding = apply(&went(
            fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
            fixture::watched_file("/etc/ssh/sshd_config", "0600", "b4"),
        ))
        .expect("fires");

        assert_eq!(finding.finding_key, "file|/etc/ssh/sshd_config");
        assert_eq!(finding.kind.as_str(), "file.changed");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(finding.subject.key["path"], "/etc/ssh/sshd_config");
    }

    #[test]
    fn a_file_whose_mode_moved_and_whose_content_did_not_is_left_to_the_rule_about_modes() {
        assert!(
            apply(&went(
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                fixture::watched_file("/etc/ssh/sshd_config", "0666", "a9"),
            ))
            .is_none(),
            "one edit that is two facts is two findings; one fact reported twice is noise"
        );
    }

    #[test]
    fn a_watched_file_that_is_gone_is_the_change_this_vocabulary_has_a_kind_for() {
        let finding = apply(&went(
            fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
            fixture::watched_file_absent("/etc/ssh/sshd_config"),
        ))
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "file.changed");
        assert!(finding.title.contains("is gone"), "{}", finding.title);
    }

    #[test]
    fn a_watched_file_that_was_put_there_between_two_readings_is_reported_too() {
        let finding = apply(&went(
            fixture::watched_file_absent("/etc/ssh/sshd_config"),
            fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
        ))
        .expect("fires");

        assert!(
            finding.title.contains("at the reading before it was not"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_first_reading_says_nothing_because_it_has_nothing_to_compare_with() {
        let change = Change::Added {
            key: "file|/etc/ssh/sshd_config".into(),
            after: fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_file_that_grew_past_the_ceiling_is_not_reported_as_one_whose_content_moved() {
        let mut huge = fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9");
        huge["sha256"] = serde_json::Value::Null;
        huge["over_the_ceiling"] = serde_json::json!(true);

        assert!(
            apply(&went(
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                huge
            ))
            .is_none(),
            "a digest that stopped being taken is said in the health of the collector; \
             reporting it as a file that changed would be a finding nobody can act on"
        );
    }
}
