use std::collections::BTreeSet;

use vigil_model::{Change, Finding, KnownKind, Severity};

use super::process_finding::{ProcessFinding, build, program_evidence};
use super::process_view::ProcessView;
use crate::{Rule, RuleContext};

pub struct UnexpectedParent;

impl Rule for UnexpectedParent {
    fn name(&self) -> &'static str {
        "unexpected_parent"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let view = ProcessView::new(after);
        if !view.is_program() || !view.is_shell() {
            return None;
        }

        let known: BTreeSet<&str> = before
            .map(|value| ProcessView::new(value).parents().into_iter().collect())
            .unwrap_or_default();
        let fresh: Vec<&str> = view
            .parents()
            .into_iter()
            .filter(|parent| !known.contains(parent))
            .filter(|parent| ProcessView::executes_what_arrives(parent))
            .collect();
        if fresh.is_empty() {
            return None;
        }

        Some(build(
            ProcessFinding {
                kind: KnownKind::ProcessUnexpectedParent,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                title: format!(
                    "{} started by {} as {}",
                    view.executable(),
                    fresh.join(", "),
                    view.user()
                ),
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: program_evidence(&view),
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
        UnexpectedParent.apply(change, &mut ctx)
    }

    #[test]
    fn a_shell_under_the_web_server_is_the_finding_this_product_exists_for() {
        let finding = apply(&Change::Added {
            key: "exec|/bin/sh|www-data".into(),
            after: fixture::program("/bin/sh", "www-data", 33, &["/usr/sbin/php-fpm8.2"]),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.kind.as_str(), "process.unexpected_parent");
        assert_eq!(finding.finding_key, "process|exec|/bin/sh|www-data");
        assert!(finding.title.contains("php-fpm"), "{}", finding.title);
    }

    #[test]
    fn a_shell_under_cron_or_systemd_is_a_host_doing_its_job() {
        for parent in [
            "/usr/sbin/cron",
            "/usr/lib/systemd/systemd",
            "/usr/bin/sshd",
        ] {
            let change = Change::Added {
                key: "exec|/bin/sh|root".into(),
                after: fixture::program("/bin/sh", "root", 0, &[parent]),
            };
            assert!(apply(&change).is_none(), "{parent}");
        }
    }

    #[test]
    fn a_parent_that_was_already_in_the_list_does_not_fire_again() {
        let before = fixture::program("/bin/sh", "www-data", 33, &["/usr/sbin/php-fpm"]);
        let mut after = fixture::program("/bin/sh", "www-data", 33, &["/usr/sbin/php-fpm"]);
        after["cmdline"] = serde_json::json!("sh -c whoami");

        assert!(
            apply(&Change::Changed {
                key: "exec|/bin/sh|www-data".into(),
                before,
                after,
            })
            .is_none()
        );
    }

    #[test]
    fn a_second_web_server_appearing_in_the_list_is_a_second_event() {
        let before = fixture::program("/bin/sh", "www-data", 33, &["/usr/sbin/php-fpm"]);
        let after = fixture::program(
            "/bin/sh",
            "www-data",
            33,
            &["/usr/sbin/nginx", "/usr/sbin/php-fpm"],
        );

        let finding = apply(&Change::Changed {
            key: "exec|/bin/sh|www-data".into(),
            before,
            after,
        })
        .expect("fires");

        assert!(finding.title.contains("nginx"), "{}", finding.title);
        assert!(
            !finding.title.contains("php-fpm"),
            "the parent that was already known is not news: {}",
            finding.title
        );
    }

    #[test]
    fn a_program_that_is_not_a_shell_is_not_this_rule() {
        let change = Change::Added {
            key: "exec|/usr/bin/python3|www-data".into(),
            after: fixture::program("/usr/bin/python3", "www-data", 33, &["/usr/sbin/nginx"]),
        };

        assert!(apply(&change).is_none());
    }
}
