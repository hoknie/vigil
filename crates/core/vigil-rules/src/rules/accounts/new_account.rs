use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use super::account_view::AccountView;
use super::second_root_account::is_new_superuser;
use crate::{Rule, RuleContext};

pub struct NewAccount;

impl Rule for NewAccount {
    fn name(&self) -> &'static str {
        "new_account"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let Change::Added { key, after } = change else {
            return None;
        };
        if !key.starts_with("account|") || is_new_superuser(change) {
            return None;
        }
        let view = AccountView::new(after);

        let severity = match view.interactive() {
            true => Severity::Medium,
            false => Severity::Low,
        };

        let mut evidence = vec![note(format!(
            "home {}, shell {}",
            view.home(),
            view.shell()
        ))];
        if let Some(state) = view.password() {
            evidence.push(note(format!("password {state}")));
        }
        if !view.shadow_readable() {
            evidence.push(note(
                "password state unknown: /etc/shadow not readable (needs root)",
            ));
        }

        Some(build(
            AccountFinding {
                kind: KnownKind::UserAccountNew,
                severity,
                rule: self.name(),
                key,
                object: "account",
                title: format!("New account {}", view.describe()),
                before: None,
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
    use crate::rules::fixture;

    fn apply(change: &Change) -> Option<Finding> {
        let mut mint = || "event-1".to_string();
        let mut ctx = RuleContext {
            now: "2026-09-09T12:04:02.104Z".into(),
            mint_event_id: &mut mint,
        };
        NewAccount.apply(change, &mut ctx)
    }

    #[test]
    fn an_account_somebody_can_log_in_to_outranks_a_service_user() {
        let person = apply(&Change::Added {
            key: "account|deploy".into(),
            after: fixture::account("deploy", 1000, "/bin/bash"),
        })
        .expect("fires");
        let service = apply(&Change::Added {
            key: "account|prometheus".into(),
            after: fixture::account("prometheus", 114, "/usr/sbin/nologin"),
        })
        .expect("fires");

        assert_eq!(person.severity, Severity::Medium);
        assert_eq!(service.severity, Severity::Low);
        assert_eq!(person.kind.as_str(), "user.account.new");
        assert_eq!(person.finding_key, "user|account|deploy");
    }

    #[test]
    fn it_stays_quiet_about_the_account_the_other_rule_speaks_about() {
        assert!(
            apply(&Change::Added {
                key: "account|toor".into(),
                after: fixture::account("toor", 0, "/bin/bash"),
            })
            .is_none()
        );
    }

    #[test]
    fn an_unreadable_shadow_is_said_out_loud_rather_than_left_blank() {
        let finding = apply(&Change::Added {
            key: "account|deploy".into(),
            after: fixture::account_without_shadow("deploy", 1000),
        })
        .expect("fires");

        assert!(
            finding
                .evidence
                .iter()
                .any(|line| line.value.contains("/etc/shadow not readable")),
            "{:?}",
            finding.evidence
        );
    }

    #[test]
    fn a_removal_is_not_a_new_account() {
        assert!(
            apply(&Change::Removed {
                key: "account|deploy".into(),
                before: fixture::account("deploy", 1000, "/bin/bash"),
            })
            .is_none()
        );
    }
}
