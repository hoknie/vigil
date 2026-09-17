use serde_json::json;
use vigil_model::{
    ControlReport, ControlTarget, Controlled, Controlling, Evidence, Finding, Kind, KnownKind,
    Rfc3339, Severity, State, Subject,
};

pub fn findings(report: &ControlReport, mint: &mut dyn FnMut() -> String) -> Vec<Finding> {
    report
        .controlled
        .iter()
        .map(|one| told(one, report, mint()))
        .collect()
}

fn told(controlled: &Controlled, report: &ControlReport, event_id: String) -> Finding {
    Finding {
        event_id,
        finding_key: format!("{}|{}", family(report.target), controlled.key),
        kind: Kind::Known(kind(report.target, controlled.done)),
        severity: match controlled.done {
            true => Severity::High,
            false => Severity::Low,
        },
        state: State::Open,
        observed_at: report.acted_at.clone(),
        first_seen_at: report.acted_at.clone(),
        occurrences: 1,
        title: title(controlled, report.controlling),
        subject: Subject {
            object: object(report.target).into(),
            key: json!({ "object": controlled.key }),
        },
        before: None,
        after: None,
        evidence: evidence(controlled, report.controlling, &report.acted_at),
        redacted: Vec::new(),
        rule: Some("console_control".into()),
        labels: Default::default(),
    }
}

fn family(target: ControlTarget) -> &'static str {
    match target {
        ControlTarget::Unit => "agent.unit.control",
        ControlTarget::Cron => "agent.cron.change",
    }
}

fn object(target: ControlTarget) -> &'static str {
    match target {
        ControlTarget::Unit => "unit",
        ControlTarget::Cron => "cron_job",
    }
}

fn kind(target: ControlTarget, done: bool) -> KnownKind {
    match (target, done) {
        (ControlTarget::Unit, true) => KnownKind::AgentUnitControlled,
        (ControlTarget::Unit, false) => KnownKind::AgentUnitControlRefused,
        (ControlTarget::Cron, true) => KnownKind::AgentCronChanged,
        (ControlTarget::Cron, false) => KnownKind::AgentCronChangeRefused,
    }
}

fn title(controlled: &Controlled, controlling: Controlling) -> String {
    match controlled.done {
        true => format!(
            "An operator asked this agent to {} {} at this console, and it did: {}",
            controlling.as_str(),
            controlled.key,
            controlled.said
        ),
        false => format!(
            "An operator asked this console to {} {} and the agent did not: {}",
            controlling.as_str(),
            controlled.key,
            controlled.said
        ),
    }
}

fn evidence(controlled: &Controlled, controlling: Controlling, at: &Rfc3339) -> Vec<Evidence> {
    let mut evidence = vec![
        said("asked", controlling.as_str()),
        said("change", controlling.said()),
    ];
    if let Some(object) = &controlled.object {
        evidence.push(said("object", object));
    }
    evidence.push(said("asked_at", at));
    evidence.push(said("outcome", &controlled.said));
    evidence.push(said("asked_from", "the local console socket"));
    evidence
}

fn said(kind: &str, value: &str) -> Evidence {
    Evidence {
        kind: kind.to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(target: ControlTarget, controlling: Controlling) -> ControlReport {
        let (done, refused) = match target {
            ControlTarget::Unit => ("unit|nginx.service", "unit|dbus.service"),
            ControlTarget::Cron => (
                "cron|/etc/crontab|root|/usr/bin/backup",
                "cron|/etc/cron.daily|root|/etc/cron.daily/logrotate",
            ),
        };
        ControlReport {
            target,
            controlling,
            acted_at: "2026-09-16T10:00:00.000Z".into(),
            controlled: vec![
                Controlled::done(done, Some("what it acted on".into()), "it did"),
                Controlled::refused(refused, "it did not, and this is why"),
            ],
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
    fn every_row_asked_for_leaves_a_finding_whether_the_agent_did_it_or_not() {
        let raised = findings(&report(ControlTarget::Unit, Controlling::Stop), &mut mint());

        assert_eq!(raised.len(), 2);
        assert_eq!(raised[0].kind.as_str(), "agent.unit.controlled");
        assert_eq!(raised[0].severity, Severity::High);
        assert_eq!(
            raised[1].kind.as_str(),
            "agent.unit.control_refused",
            "a service asked to stop and still running is the same question at an incident \
             as one that stopped"
        );
        assert_eq!(raised[1].severity, Severity::Low);
    }

    #[test]
    fn a_crontab_edited_from_the_console_is_a_kind_of_its_own_and_not_a_unit() {
        let raised = findings(
            &report(ControlTarget::Cron, Controlling::Comment),
            &mut mint(),
        );

        assert_eq!(raised[0].kind.as_str(), "agent.cron.changed");
        assert_eq!(raised[1].kind.as_str(), "agent.cron.change_refused");
        assert_eq!(raised[0].subject.object, "cron_job");
        assert_eq!(
            raised[0].finding_key, "agent.cron.change|cron|/etc/crontab|root|/usr/bin/backup",
            "it is keyed by the row it changed, so asking twice raises a counter and not a \
             second finding"
        );
    }

    #[test]
    fn the_finding_says_what_was_asked_in_words_and_by_which_road_it_arrived() {
        let raised = findings(&report(ControlTarget::Unit, Controlling::Mask), &mut mint());

        assert_eq!(raised[0].rule.as_deref(), Some("console_control"));
        let evidence = format!("{:?}", raised[0].evidence);
        assert!(evidence.contains("the local console socket"), "{evidence}");
        assert!(evidence.contains("mask"), "{evidence}");
        assert!(
            evidence.contains("until it is unmasked"),
            "the word alone is jargon in a journal read a year later: {evidence}"
        );
        assert!(raised[1].title.contains("did not"), "{}", raised[1].title);
    }
}
