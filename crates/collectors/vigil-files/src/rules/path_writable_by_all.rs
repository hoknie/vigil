use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::file_finding::{FileFinding, build, standing};
use crate::types::{Family, FileView};
use vigil_rules::{Rule, RuleContext};

pub struct PathWritableByAll;

impl Rule for PathWritableByAll {
    fn name(&self) -> &'static str {
        "path_writable_by_all"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };

        let now = FileView::new(key, after);
        if !now.is(Family::Directory) || !now.present() || !now.writable_by_anyone() {
            return None;
        }
        if let Some(before) = before
            && FileView::new(key, before).writable_by_anyone()
        {
            return None;
        }

        Some(build(
            FileFinding {
                kind: KnownKind::FilePathWritableByAll,
                severity: Severity::High,
                rule: self.name(),
                key,
                object: "directory",
                title: format!(
                    "{} is on the path this host runs programs from and is mode {}: any account can put a program there and wait for somebody to type its name",
                    now.path(),
                    now.mode().unwrap_or("unknown")
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    standing(&now),
                    Evidence {
                        kind: "note".into(),
                        value: "the sticky bit is what makes a directory safe to share, and this one does not carry it".into(),
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
        PathWritableByAll.apply(change, &mut ctx)
    }

    fn appeared(mode: &str) -> Change {
        Change::Added {
            key: "directory|/usr/local/bin".into(),
            after: fixture::watched_directory("/usr/local/bin", mode),
        }
    }

    #[test]
    fn a_directory_of_the_path_anybody_may_write_into_is_reported_the_first_time_it_is_seen() {
        let finding = apply(&appeared("0777")).expect("fires");

        assert_eq!(finding.finding_key, "directory|/usr/local/bin");
        assert_eq!(finding.kind.as_str(), "file.path_writable_by_all");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.subject.object, "directory");
    }

    #[test]
    fn a_directory_of_the_path_as_every_host_ships_it_says_nothing() {
        assert!(apply(&appeared("0755")).is_none());
        assert!(apply(&appeared("0775")).is_none());
    }

    #[test]
    fn a_shared_directory_with_the_sticky_bit_is_not_one_anybody_can_plant_a_program_in() {
        assert!(apply(&appeared("1777")).is_none());
    }

    #[test]
    fn a_directory_that_was_already_open_is_not_reported_again_when_its_owner_moves() {
        let mut after = fixture::watched_directory("/usr/local/bin", "0777");
        after["uid"] = serde_json::json!(1000);
        let change = Change::Changed {
            key: "directory|/usr/local/bin".into(),
            before: fixture::watched_directory("/usr/local/bin", "0777"),
            after,
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_watched_file_is_never_a_directory_of_the_path() {
        let change = Change::Added {
            key: "file|/etc/hosts".into(),
            after: fixture::watched_file("/etc/hosts", "0666", "a9"),
        };

        assert!(apply(&change).is_none());
    }
}
