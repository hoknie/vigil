use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use super::account_view::AccountView;
use super::second_root_account::is_new_superuser;
use crate::{Rule, RuleContext};

pub struct AccountUnlocked;

pub fn is_unlock(change: &Change) -> bool {
    let Change::Changed { key, before, after } = change else {
        return false;
    };
    if !key.starts_with("account|") {
        return false;
    }
    let (was, now) = (AccountView::new(before), AccountView::new(after));

    was.shadow_readable()
        && now.shadow_readable()
        && !was.password_permits_login()
        && now.password_permits_login()
}

impl Rule for AccountUnlocked {
    fn name(&self) -> &'static str {
        "account_unlocked"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        if !is_unlock(change) || is_new_superuser(change) {
            return None;
        }
        let Change::Changed { key, before, after } = change else {
            return None;
        };
        let (was, now) = (AccountView::new(before), AccountView::new(after));

        let severity = match (now.password() == Some("empty"), now.is_superuser()) {
            (true, _) | (_, true) => Severity::Critical,
            _ => Severity::High,
        };

        Some(build(
            AccountFinding {
                kind: KnownKind::UserAccountUnlocked,
                severity,
                rule: self.name(),
                key,
                object: "account",
                title: format!(
                    "Account {} can be logged into again (was {}, now {})",
                    now.name(),
                    was.password().unwrap_or("?"),
                    now.password().unwrap_or("?")
                ),
                before: Some(before.clone()),
                after: Some(after.clone()),
                evidence: vec![
                    note(format!("shell {}", now.shell())),
                    note(match now.password() {
                        Some("empty") => {
                            "no password at all; a login with nothing to guess where the \
                             host permits empty passwords"
                                .to_string()
                        }
                        _ => format!(
                            "password state {} to {}; the password itself is not read",
                            was.password().unwrap_or("?"),
                            now.password().unwrap_or("?")
                        ),
                    }),
                ],
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
        AccountUnlocked.apply(change, &mut ctx)
    }

    #[test]
    fn a_locked_account_that_can_be_logged_into_again_is_reported() {
        let finding = apply(&Change::Changed {
            key: "account|backup".into(),
            before: fixture::account_with_password("backup", 34, "locked", 18000),
            after: fixture::account_with_password("backup", 34, "set", 19200),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.account.unlocked");
        assert_eq!(finding.severity, Severity::High);
        assert_eq!(finding.finding_key, "user|account|backup");
    }

    #[test]
    fn an_unlock_that_leaves_no_password_at_all_is_worse_than_one_that_sets_a_new_one() {
        let finding = apply(&Change::Changed {
            key: "account|backup".into(),
            before: fixture::account_with_password("backup", 34, "locked", 18000),
            after: fixture::account_with_password("backup", 34, "empty", 19200),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert!(
            finding
                .evidence
                .iter()
                .any(|line| line.value.contains("nothing to guess")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn gaining_the_privilege_to_read_shadow_is_not_an_unlock() {
        let change = Change::Changed {
            key: "account|backup".into(),
            before: fixture::account_without_shadow("backup", 34),
            after: fixture::account_with_password("backup", 34, "set", 19200),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn locking_an_account_is_not_unlocking_it() {
        let change = Change::Changed {
            key: "account|deploy".into(),
            before: fixture::account_with_password("deploy", 1000, "set", 19000),
            after: fixture::account_with_password("deploy", 1000, "locked", 19000),
        };

        assert!(apply(&change).is_none());
    }
}
