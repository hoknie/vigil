use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct PreloadChanged;

impl Rule for PreloadChanged {
    fn name(&self) -> &'static str {
        "preload_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Preload) {
            return None;
        }

        let was = before.map(|value| PersistenceView::new(key, value));
        let title = match (was.as_ref().map(|view| view.present()), view.present()) {
            (Some(false) | None, true) => format!(
                "{} now exists and preloads {}",
                view.path(),
                describe(&view.entries())
            ),
            (Some(true), false) => format!("{} was removed", view.path()),
            _ if !view.readable() => format!("{} exists and is no longer readable", view.path()),
            _ => format!("{} now preloads {}", view.path(), describe(&view.entries())),
        };

        let mut evidence = vec![Evidence {
            kind: "path".into(),
            value: view.path().to_string(),
        }];
        if let Some(was) = &was {
            evidence.push(Evidence {
                kind: "note".into(),
                value: format!("was: {}", describe(&was.entries())),
            });
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistencePreloadChanged,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                object: "preload",
                title,
                before: before.cloned(),
                after: Some(after.clone()),
                evidence,
            },
            ctx,
        ))
    }
}

fn describe(entries: &[&str]) -> String {
    match entries.is_empty() {
        true => "nothing".to_string(),
        false => entries.join(", "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        PreloadChanged.apply(change, &mut ctx)
    }

    #[test]
    fn a_file_that_did_not_exist_and_now_preloads_a_library_is_critical() {
        let finding = apply(&Change::Changed {
            key: "preload|/etc/ld.so.preload".into(),
            before: fixture::preload(&[]),
            after: fixture::preload(&["/lib/libprocesshider.so"]),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.kind.as_str(), "persistence.preload_changed");
        assert_eq!(
            finding.finding_key,
            "persistence|preload|/etc/ld.so.preload"
        );
        assert!(
            finding.title.contains("/lib/libprocesshider.so"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_library_taken_out_of_the_list_is_reported_too() {
        let finding = apply(&Change::Changed {
            key: "preload|/etc/ld.so.preload".into(),
            before: fixture::preload(&["/lib/libprocesshider.so"]),
            after: fixture::preload(&[]),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert!(
            finding
                .evidence
                .iter()
                .any(|e| e.value.contains("libprocesshider")),
            "the reader has to be able to see what was there: {:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_shell_profile_is_somebody_elses_rule() {
        let change = Change::Changed {
            key: "script|/etc/profile".into(),
            before: fixture::script("/etc/profile", "profile", "aaa"),
            after: fixture::script("/etc/profile", "profile", "bbb"),
        };

        assert!(apply(&change).is_none());
    }
}
