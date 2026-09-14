use serde_json::json;
use vigil_model::{Changed, Evidence, Finding, Kind, KnownKind, Rfc3339, Severity, State, Subject};

use super::carried::Carried;

pub fn findings(carried: &Carried, mint: &mut dyn FnMut() -> String) -> Vec<Finding> {
    carried
        .report
        .changed
        .iter()
        .zip(carried.asked.iter())
        .map(|(changed, asked)| told(changed, asked, &carried.report.acted_at, mint()))
        .collect()
}

fn told(changed: &Changed, asked: &str, at: &Rfc3339, event_id: String) -> Finding {
    Finding {
        event_id,
        finding_key: format!("agent.account.change|{}", changed.key),
        kind: Kind::Known(match changed.done {
            true => KnownKind::AgentAccountChanged,
            false => KnownKind::AgentAccountChangeRefused,
        }),
        severity: match changed.done {
            true => Severity::High,
            false => Severity::Low,
        },
        state: State::Open,
        observed_at: at.clone(),
        first_seen_at: at.clone(),
        occurrences: 1,
        title: match changed.done {
            true => format!(
                "An operator changed {} at this console: {asked}",
                changed.key
            ),
            false => format!(
                "An operator asked this console to {asked} and the agent did not: {}",
                changed.said
            ),
        },
        subject: Subject {
            object: changed.object.as_str().into(),
            key: json!({ "object": changed.key }),
        },
        before: None,
        after: None,
        evidence: vec![
            said("asked", changed.changing.as_str()),
            said("change", asked),
            said("asked_at", at),
            said("outcome", &changed.said),
            said("asked_from", "the local console socket"),
        ],
        redacted: Vec::new(),
        rule: Some("console_change".into()),
        labels: Default::default(),
    }
}

fn said(kind: &str, value: &str) -> Evidence {
    Evidence {
        kind: kind.to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::{AccountChange, ChangeReport};

    use super::*;

    fn carried() -> Carried {
        let delete = AccountChange::DeleteUser {
            name: "contractor".into(),
        };
        let root = AccountChange::DeleteUser {
            name: "root".into(),
        };
        Carried {
            report: ChangeReport {
                acted_at: "2026-09-14T10:00:00.000Z".into(),
                changed: vec![
                    Changed::done(&delete, "userdel finished"),
                    Changed::refused(&root, "root is uid 0"),
                ],
            },
            asked: vec![delete.said(), root.said()],
        }
    }

    fn mint() -> impl FnMut() -> String {
        let mut at = 0;
        move || {
            at += 1;
            format!("event-{at}")
        }
    }

    #[test]
    fn every_change_asked_for_leaves_a_finding_whether_the_agent_made_it_or_not() {
        let raised = findings(&carried(), &mut mint());

        assert_eq!(raised.len(), 2);
        assert_eq!(raised[0].kind.as_str(), "agent.account.changed");
        assert_eq!(raised[0].severity, Severity::High);
        assert_eq!(
            raised[1].kind.as_str(),
            "agent.account.change_refused",
            "a deletion asked for and not made is the same question at an incident as one \
             that was"
        );
        assert_eq!(raised[1].severity, Severity::Low);
    }

    #[test]
    fn the_finding_is_keyed_by_the_row_it_changed_and_says_who_asked_by_which_road() {
        let raised = findings(&carried(), &mut mint());

        assert_eq!(
            raised[0].finding_key,
            "agent.account.change|account|contractor"
        );
        assert_eq!(raised[0].subject.object, "user");
        assert_eq!(raised[0].rule.as_deref(), Some("console_change"));
        assert!(
            raised[0].title.starts_with("An operator"),
            "{}",
            raised[0].title
        );
        let evidence = format!("{:?}", raised[0].evidence);
        assert!(evidence.contains("the local console socket"), "{evidence}");
        assert!(evidence.contains("delete"), "{evidence}");
        assert!(
            evidence.contains("keeping its home directory"),
            "the change is written in words beside the outcome: {evidence}"
        );
        assert!(
            raised[1].title.contains("root is uid 0"),
            "{}",
            raised[1].title
        );
    }
}
