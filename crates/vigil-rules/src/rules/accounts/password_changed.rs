use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use super::account_unlocked::is_unlock;
use super::account_view::AccountView;
use super::second_root_account::is_new_superuser;
use crate::{Rule, RuleContext};

pub struct PasswordChanged;

impl Rule for PasswordChanged {
    fn name(&self) -> &'static str {
        "password_changed"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        if !key.starts_with("account|") || is_unlock(change) || is_new_superuser(change) {
            return None;
        }

        let (was, now) = (AccountView::new(before), AccountView::new(after));
        if !was.shadow_readable() || !now.shadow_readable() {
            return None;
        }

        let (Some(before_day), Some(after_day)) = (was.last_change_day(), now.last_change_day())
        else {
            return None;
        };
        if before_day == after_day {
            return None;
        }

        let severity = match now.is_superuser() {
            true => Severity::High,
            false => Severity::Medium,
        };

        Some(build(
            AccountFinding {
                kind: KnownKind::UserPasswordChanged,
                severity,
                rule: self.name(),
                key,
                object: "account",
                title: format!("The password of {} was changed", now.name()),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![note(format!(
                    "the recorded day of the last password change moved from {before_day} to \
                     {after_day} (days since 1970-01-01); the password itself is never read"
                ))],
            },
            ctx,
        ))
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
        PasswordChanged.apply(change, &mut ctx)
    }

    #[test]
    fn a_date_that_moved_is_the_whole_evidence() {
        let finding = apply(&Change::Changed {
            key: "account|deploy".into(),
            before: fixture::account_with_password("deploy", 1000, "set", 19000),
            after: fixture::account_with_password("deploy", 1000, "set", 19200),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.password.changed");
        assert_eq!(finding.severity, Severity::Medium);
        assert!(
            finding.evidence[0].value.contains("19000")
                && finding.evidence[0].value.contains("19200"),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn roots_password_outranks_anybody_elses() {
        let finding = apply(&Change::Changed {
            key: "account|root".into(),
            before: fixture::account_with_password("root", 0, "set", 19000),
            after: fixture::account_with_password("root", 0, "set", 19200),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn an_unlock_belongs_to_the_other_rule_even_though_the_date_moved_too() {
        let change = Change::Changed {
            key: "account|backup".into(),
            before: fixture::account_with_password("backup", 34, "locked", 18000),
            after: fixture::account_with_password("backup", 34, "set", 19200),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_shadow_that_became_readable_did_not_change_anybodys_password() {
        let change = Change::Changed {
            key: "account|deploy".into(),
            before: fixture::account_without_shadow("deploy", 1000),
            after: fixture::account_with_password("deploy", 1000, "set", 19200),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_change_to_something_else_on_the_account_is_not_a_password_change() {
        let mut after = fixture::account_with_password("deploy", 1000, "set", 19000);
        after["shell"] = serde_json::json!("/bin/zsh");
        let change = Change::Changed {
            key: "account|deploy".into(),
            before: fixture::account_with_password("deploy", 1000, "set", 19000),
            after,
        };

        assert!(apply(&change).is_none());
    }
}
