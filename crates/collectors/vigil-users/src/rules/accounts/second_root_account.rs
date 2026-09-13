use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use crate::types::AccountView;
use vigil_rules::{Rule, RuleContext};

pub struct SecondRootAccount;

pub fn is_new_superuser(change: &Change) -> bool {
    match change {
        Change::Added { key, after } if key.starts_with("account|") => {
            let view = AccountView::new(after);
            view.is_superuser() && view.name() != "root"
        }
        Change::Changed { key, before, after } if key.starts_with("account|") => {
            let (was, now) = (AccountView::new(before), AccountView::new(after));
            now.is_superuser() && !was.is_superuser() && now.name() != "root"
        }
        _ => false,
    }
}

impl Rule for SecondRootAccount {
    fn name(&self) -> &'static str {
        "second_root_account"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        if !is_new_superuser(change) {
            return None;
        }

        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        let view = AccountView::new(after);

        let title = match before {
            Some(_) => format!("Account {} moved to uid 0: a second superuser", view.name()),
            None => format!("A second account with uid 0: {}", view.name()),
        };
        let evidence = vec![
            note(format!(
                "uid 0 is the superuser whatever the name; this account is {}",
                view.name()
            )),
            note(format!("home {}, shell {}", view.home(), view.shell())),
        ];

        Some(build(
            AccountFinding {
                kind: KnownKind::UserAccountUid0,
                severity: Severity::Critical,
                rule: self.name(),
                key,
                object: "account",
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
        SecondRootAccount.apply(change, &mut ctx)
    }

    #[test]
    fn an_account_created_with_uid_zero_is_the_loudest_thing_this_build_says() {
        let finding = apply(&Change::Added {
            key: "account|toor".into(),
            after: fixture::account("toor", 0, "/bin/bash"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.account.uid0");
        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.finding_key, "user|account|toor");
        assert!(
            finding.title.contains("second account with uid 0"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn an_existing_account_moved_to_uid_zero_is_the_same_event_by_another_route() {
        let finding = apply(&Change::Changed {
            key: "account|backup".into(),
            before: fixture::account("backup", 34, "/bin/sh"),
            after: fixture::account("backup", 0, "/bin/sh"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
        assert!(finding.before.is_some() && finding.after.is_some());
        assert!(
            finding.title.contains("moved to uid 0"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn root_itself_is_not_a_second_root() {
        assert!(
            apply(&Change::Added {
                key: "account|root".into(),
                after: fixture::account("root", 0, "/bin/bash"),
            })
            .is_none()
        );
    }

    #[test]
    fn an_ordinary_new_account_is_not_its_business() {
        assert!(
            apply(&Change::Added {
                key: "account|deploy".into(),
                after: fixture::account("deploy", 1000, "/bin/bash"),
            })
            .is_none()
        );
    }
}
