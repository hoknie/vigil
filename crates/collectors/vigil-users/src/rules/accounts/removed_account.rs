use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use crate::types::AccountView;
use vigil_rules::{Rule, RuleContext};

pub struct RemovedAccount;

impl Rule for RemovedAccount {
    fn name(&self) -> &'static str {
        "removed_account"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Removed { key, before } = change else {
            return None;
        };
        if !key.starts_with("account|") {
            return None;
        }
        let view = AccountView::new(before);

        let severity = match view.is_superuser() {
            true => Severity::High,
            false => Severity::Medium,
        };

        Some(build(
            AccountFinding {
                kind: KnownKind::UserAccountRemoved,
                severity,
                rule: self.name(),
                key,
                object: "account",
                title: format!("Account {} is gone", view.describe()),
                before: Some(before.clone()),
                after: None,
                evidence: vec![note(format!(
                    "home {}, shell {}",
                    view.home(),
                    view.shell()
                ))],
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
        RemovedAccount.apply(change, &mut ctx)
    }

    #[test]
    fn reports_the_disappearance_with_what_the_account_used_to_be() {
        let finding = apply(&Change::Removed {
            key: "account|deploy".into(),
            before: fixture::account("deploy", 1000, "/bin/bash"),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.account.removed");
        assert_eq!(finding.severity, Severity::Medium);
        assert_eq!(
            finding.finding_key, "user|account|deploy",
            "it has to be the same key the arrival carried, or the store cannot close it"
        );
        assert!(finding.before.is_some() && finding.after.is_none());
    }

    #[test]
    fn a_superuser_that_disappears_outranks_an_ordinary_one() {
        let finding = apply(&Change::Removed {
            key: "account|toor".into(),
            before: fixture::account("toor", 0, "/bin/bash"),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn an_addition_is_not_a_removal() {
        assert!(
            apply(&Change::Added {
                key: "account|deploy".into(),
                after: fixture::account("deploy", 1000, "/bin/bash"),
            })
            .is_none()
        );
    }
}
