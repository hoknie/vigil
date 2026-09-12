use vigil_model::{Change, Finding, KnownKind, Severity};

use super::account_finding::{AccountFinding, build, note};
use super::sudoer_view::SudoerView;
use crate::{Rule, RuleContext};

pub struct SudoGrantAdded;

impl Rule for SudoGrantAdded {
    fn name(&self) -> &'static str {
        "sudo_grant_added"
    }

    fn apply(&self, change: &Change, ctx: &mut RuleContext<'_>) -> Option<Finding> {
        let (key, before, after) = match change {
            Change::Added { key, after } => (key, None, after),
            Change::Changed { key, before, after } => (key, Some(before), after),
            Change::Removed { .. } => return None,
        };
        if !key.starts_with("sudoer|") {
            return None;
        }

        let now = SudoerView::new(after);
        let widened = match before {
            None => true,
            Some(before) => {
                let was = SudoerView::new(before);
                (now.nopasswd() && !was.nopasswd()) || (now.all_commands() && !was.all_commands())
            }
        };
        if !widened {
            return None;
        }

        let severity = match now.is_unrestricted() {
            true => Severity::Critical,
            false => Severity::High,
        };

        let subject = match now.is_group() {
            true => format!("Members of {}", now.who()),
            false => now.who().to_string(),
        };

        Some(build(
            AccountFinding {
                kind: KnownKind::UserGroupPrivilegedMemberAdded,
                severity,
                rule: self.name(),
                key,
                object: "sudoers",
                title: match now.is_unrestricted() {
                    true => {
                        format!("{subject} may now run any command as any user without a password")
                    }
                    false => format!("{subject} may now run commands as another user"),
                },
                before: before.cloned(),
                after: Some(after.clone()),
                evidence: vec![
                    note(format!("granted by {}", now.sources().join(", "))),
                    note(now.specs().join(" · ")),
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
        SudoGrantAdded.apply(change, &mut ctx)
    }

    #[test]
    fn a_blanket_passwordless_grant_is_the_loudest_shape_of_this_finding() {
        let finding = apply(&Change::Added {
            key: "sudoer|deploy".into(),
            after: fixture::sudoer("deploy", "ALL=(ALL) NOPASSWD: ALL", true, true),
        })
        .expect("fires");

        assert_eq!(finding.kind.as_str(), "user.group.privileged_member_added");
        assert_eq!(finding.severity, Severity::Critical);
        assert_eq!(finding.finding_key, "user|sudoer|deploy");
        assert!(
            finding.title.contains("without a password"),
            "{}",
            finding.title
        );
    }

    #[test]
    fn a_grant_for_one_command_is_reported_but_does_not_shout() {
        let finding = apply(&Change::Added {
            key: "sudoer|backup".into(),
            after: fixture::sudoer("backup", "ALL=(root) /usr/bin/rsync", false, false),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::High);
    }

    #[test]
    fn a_grant_that_gained_nopasswd_is_a_new_grant_in_the_way_that_matters() {
        let finding = apply(&Change::Changed {
            key: "sudoer|deploy".into(),
            before: fixture::sudoer("deploy", "ALL=(ALL) ALL", false, true),
            after: fixture::sudoer("deploy", "ALL=(ALL) NOPASSWD: ALL", true, true),
        })
        .expect("fires");

        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn editing_the_command_list_of_an_existing_grant_is_not_a_new_grant() {
        let change = Change::Changed {
            key: "sudoer|backup".into(),
            before: fixture::sudoer("backup", "ALL=(root) /usr/bin/rsync", false, false),
            after: fixture::sudoer("backup", "ALL=(root) /usr/bin/restic", false, false),
        };

        assert!(apply(&change).is_none());
    }

    #[test]
    fn a_grant_being_taken_away_is_not_a_grant_being_made() {
        let change = Change::Removed {
            key: "sudoer|deploy".into(),
            before: fixture::sudoer("deploy", "ALL=(ALL) NOPASSWD: ALL", true, true),
        };

        assert!(apply(&change).is_none());
    }
}
