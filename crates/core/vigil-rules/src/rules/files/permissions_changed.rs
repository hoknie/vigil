use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::file_finding::{FileFinding, build, standing};
use super::file_view::{Family, FileView};
use crate::{Rule, RuleContext};

pub struct FilePermissionsChanged;

impl Rule for FilePermissionsChanged {
    fn name(&self) -> &'static str {
        "file_permissions_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let was = FileView::new(key, before);
        let now = FileView::new(key, after);
        if now.family().is_none() || !was.present() || !now.present() {
            return None;
        }
        if was.mode() == now.mode() && was.owner() == now.owner() {
            return None;
        }
        if !was.runs_as_its_owner() && now.runs_as_its_owner() {
            return None;
        }
        if now.is(Family::Directory) && !was.writable_by_anyone() && now.writable_by_anyone() {
            return None;
        }

        Some(build(
            FileFinding {
                kind: KnownKind::FilePermissionsChanged,
                severity: Severity::Medium,
                rule: self.name(),
                key,
                object: object(&now),
                title: format!(
                    "{} is now mode {} owned by {}:{}, and it was mode {} owned by {}:{}",
                    now.path(),
                    shown(now.mode()),
                    number(now.owner().0),
                    number(now.owner().1),
                    shown(was.mode()),
                    number(was.owner().0),
                    number(was.owner().1)
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    standing(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "the content of it is the same as it was: this is who may read it and who may write it, and nothing else".into(),
                    },
                ],
            },
            ctx,
        ))
    }
}

fn object(view: &FileView<'_>) -> &'static str {
    match view.is(Family::Directory) {
        true => "directory",
        false => "file",
    }
}

fn shown(mode: Option<&str>) -> &str {
    mode.unwrap_or("unknown")
}

fn number(who: Option<u64>) -> String {
    who.map(|who| who.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-11T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        FilePermissionsChanged.apply(change, &mut ctx)
    }

    fn went(before: serde_json::Value, after: serde_json::Value) -> Change {
        Change::Changed {
            key: "file|/etc/ssh/sshd_config".into(),
            before,
            after,
        }
    }

    #[test]
    fn a_file_anybody_may_write_to_now_and_could_not_before_is_reported() {
        let finding = apply(&went(
            fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
            fixture::watched_file("/etc/ssh/sshd_config", "0666", "a9"),
        ))
        .expect("fires");

        assert_eq!(finding.finding_key, "file|/etc/ssh/sshd_config");
        assert_eq!(finding.kind.as_str(), "file.permissions_changed");
        assert!(finding.title.contains("0666"), "{}", finding.title);
    }

    #[test]
    fn an_owner_that_changed_under_the_same_mode_is_the_same_finding() {
        let mut after = fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9");
        after["uid"] = serde_json::json!(1000);

        assert!(
            apply(&went(
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                after
            ))
            .is_some(),
            "a file handed to another account is as much a change of who may write it as a \
             mode is, and the mode alone would not say it happened"
        );
    }

    #[test]
    fn a_file_that_became_one_running_as_its_owner_is_left_to_the_rule_about_that() {
        assert!(
            apply(&went(
                fixture::watched_file("/usr/bin/at", "0755", "a9"),
                fixture::watched_file("/usr/bin/at", "4755", "a9"),
            ))
            .is_none(),
            "a new setuid bit is the sharper of the two facts and has a kind of its own; \
             reporting both would put two findings on one chmod"
        );
    }

    #[test]
    fn a_directory_of_the_path_thrown_open_is_left_to_the_rule_about_that() {
        let change = Change::Changed {
            key: "directory|/usr/local/bin".into(),
            before: fixture::watched_directory("/usr/local/bin", "0755"),
            after: fixture::watched_directory("/usr/local/bin", "0777"),
        };

        assert!(
            apply(&change).is_none(),
            "a directory of the path anybody may write into is the sharper of the two facts \
             and has a kind of its own; reporting both would put two findings on one chmod"
        );
    }

    #[test]
    fn a_directory_of_the_path_whose_owner_moved_is_still_a_finding_of_this_rule() {
        let mut after = fixture::watched_directory("/usr/local/bin", "0755");
        after["uid"] = serde_json::json!(1000);
        let change = Change::Changed {
            key: "directory|/usr/local/bin".into(),
            before: fixture::watched_directory("/usr/local/bin", "0755"),
            after,
        };

        assert!(apply(&change).is_some());
    }

    #[test]
    fn a_file_whose_content_moved_and_whose_mode_did_not_says_nothing_here() {
        assert!(
            apply(&went(
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "b4"),
            ))
            .is_none()
        );
    }

    #[test]
    fn a_file_that_is_not_there_on_either_reading_is_never_a_mode_that_changed() {
        assert!(
            apply(&went(
                fixture::watched_file("/etc/ssh/sshd_config", "0600", "a9"),
                fixture::watched_file_absent("/etc/ssh/sshd_config"),
            ))
            .is_none(),
            "a file that is gone has no mode, and the reading that says it is gone is the one \
             a reader needs rather than a mode that went from 0600 to nothing"
        );
    }
}
