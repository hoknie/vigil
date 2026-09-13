use vigil_model::{Change, Evidence, Finding, KnownKind, Severity};

use super::persistence_finding::{PersistenceFinding, build};
use super::persistence_view::{Family, PersistenceView};
use crate::{Rule, RuleContext};

pub struct ShellProfileChanged;

impl Rule for ShellProfileChanged {
    fn name(&self) -> &'static str {
        "shell_profile_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Added { key, after } => (key, None, after),
            Change::Removed { .. } => return None,
        };
        let view = PersistenceView::new(key, after);
        if !view.is(Family::Script) {
            return None;
        }
        if !view.present() {
            return None;
        }

        let was = before.map(|value| PersistenceView::new(key, value));
        if let Some(was) = &was
            && was.present()
            && was.digest() == view.digest()
            && was.mode() == view.mode()
            && was.owner() == view.owner()
        {
            return None;
        }

        let boot = view.script_family() == "boot";
        let what = match boot {
            true => "Boot script",
            false => "Shell profile",
        };
        let title = match was.as_ref().map(|was| was.present()) {
            Some(true) => format!("{what} {} changed", view.path()),
            _ => format!("{what} {} appeared", view.path()),
        };

        let mut evidence = vec![Evidence {
            kind: "path".into(),
            value: format!(
                "{} ({} owned by {})",
                view.path(),
                view.mode(),
                view.owner().0
            ),
        }];
        if !view.readable() {
            evidence.push(Evidence {
                kind: "note".into(),
                value: "the file exists and is not readable; only its metadata is known".into(),
            });
        }

        Some(build(
            PersistenceFinding {
                kind: KnownKind::PersistenceShellProfileChanged,
                severity: match boot {
                    true => Severity::High,
                    false => Severity::Medium,
                },
                rule: self.name(),
                key,
                object: "script",
                title,
                before: before.cloned(),
                after: Some(after.clone()),
                evidence,
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
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        ShellProfileChanged.apply(change, &mut ctx)
    }

    #[test]
    fn a_boot_script_outranks_a_login_profile() {
        let profile = apply(&Change::Changed {
            key: "script|/etc/profile".into(),
            before: fixture::script("/etc/profile", "profile", "aaa"),
            after: fixture::script("/etc/profile", "profile", "bbb"),
        })
        .expect("fires");
        let boot = apply(&Change::Changed {
            key: "script|/etc/rc.local".into(),
            before: fixture::script("/etc/rc.local", "boot", "aaa"),
            after: fixture::script("/etc/rc.local", "boot", "bbb"),
        })
        .expect("fires");

        assert_eq!(profile.severity, Severity::Medium);
        assert_eq!(boot.severity, Severity::High);
        assert!(boot.title.starts_with("Boot script"), "{}", boot.title);
        assert_eq!(profile.finding_key, "persistence|script|/etc/profile");
    }

    #[test]
    fn a_file_that_was_not_there_and_still_is_not_has_not_changed() {
        let mut absent = fixture::script("/etc/zsh/zshrc", "profile", "aaa");
        absent["present"] = serde_json::json!(false);
        absent["sha256"] = serde_json::Value::Null;
        let mut other = absent.clone();
        other["size"] = serde_json::json!(1);

        assert!(
            apply(&Change::Changed {
                key: "script|/etc/zsh/zshrc".into(),
                before: absent,
                after: other,
            })
            .is_none()
        );
    }

    #[test]
    fn a_dormant_file_made_executable_is_a_change_even_though_its_bytes_are_not() {
        let before = fixture::script("/etc/profile.d/x.sh", "profile", "aaa");
        let mut after = before.clone();
        after["mode"] = serde_json::json!("0755");

        let finding = apply(&Change::Changed {
            key: "script|/etc/profile.d/x.sh".into(),
            before,
            after,
        })
        .expect("fires");

        assert!(finding.title.contains("changed"), "{}", finding.title);
    }

    #[test]
    fn a_reading_that_differs_in_nothing_a_person_would_call_a_change_is_silent() {
        let before = fixture::script("/etc/profile", "profile", "aaa");
        let mut after = before.clone();
        after["size"] = serde_json::json!(4096);

        assert!(
            apply(&Change::Changed {
                key: "script|/etc/profile".into(),
                before,
                after,
            })
            .is_none()
        );
    }
}
