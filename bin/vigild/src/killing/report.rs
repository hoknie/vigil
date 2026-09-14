use serde_json::json;
use vigil_model::{
    Evidence, Finding, KillReport, Killed, Killing, Kind, KnownKind, Rfc3339, Severity, State,
    Subject,
};

use super::targets::Target;

pub fn killed(target: &Target, said: &str) -> Killed {
    Killed::done(target.key.clone(), target.pid, target.program.clone(), said)
}

pub fn findings(report: &KillReport, mint: &mut dyn FnMut() -> String) -> Vec<Finding> {
    report
        .killed
        .iter()
        .map(|one| told(one, report, mint()))
        .collect()
}

fn told(killed: &Killed, report: &KillReport, event_id: String) -> Finding {
    Finding {
        event_id,
        finding_key: format!("agent.socket.kill|{}", killed.key),
        kind: Kind::Known(match killed.done {
            true => KnownKind::AgentSocketKilled,
            false => KnownKind::AgentSocketKillRefused,
        }),
        severity: match killed.done {
            true => Severity::High,
            false => Severity::Low,
        },
        state: State::Open,
        observed_at: report.acted_at.clone(),
        first_seen_at: report.acted_at.clone(),
        occurrences: 1,
        title: title(killed, report.killing),
        subject: Subject {
            object: "socket".into(),
            key: json!({ "object": killed.key }),
        },
        before: None,
        after: None,
        evidence: evidence(killed, report.killing, &report.acted_at),
        redacted: Vec::new(),
        rule: Some("console_kill".into()),
        labels: Default::default(),
    }
}

fn title(killed: &Killed, killing: Killing) -> String {
    let who = match (&killed.program, killed.pid) {
        (Some(program), Some(pid)) => format!("{program} (pid {pid})"),
        (None, Some(pid)) => format!("pid {pid}"),
        _ => "an unidentified process".to_string(),
    };
    match killed.done {
        true => format!(
            "An operator closed {} at this console: {} on {who}",
            killed.key,
            killing.said()
        ),
        false => format!(
            "An operator asked this console to close {} and the agent did not: {}",
            killed.key, killed.said
        ),
    }
}

fn evidence(killed: &Killed, killing: Killing, at: &Rfc3339) -> Vec<Evidence> {
    let mut evidence = vec![
        said("asked", killing.as_str().to_string()),
        said("asked_at", at.clone()),
        said("outcome", killed.said.clone()),
        said("asked_from", "the local console socket".to_string()),
    ];
    if let Some(pid) = killed.pid {
        evidence.push(said("pid", pid.to_string()));
    }
    if let Some(program) = &killed.program {
        evidence.push(said("program", program.clone()));
    }
    evidence
}

fn said(kind: &str, value: String) -> Evidence {
    Evidence {
        kind: kind.to_string(),
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(killed: Vec<Killed>) -> KillReport {
        KillReport {
            killing: Killing::Terminate,
            acted_at: "2026-09-14T10:00:00.000Z".into(),
            killed,
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
    fn everything_the_agent_did_to_this_host_leaves_a_finding_whether_it_worked_or_not() {
        let mut mint = mint();
        let raised = findings(
            &report(vec![
                Killed::done(
                    "tcp|0.0.0.0:4444",
                    30211,
                    Some("/tmp/.x/nc".into()),
                    "SIGTERM sent",
                ),
                Killed::refused("tcp|0.0.0.0:22", "pid 1 is not signalled"),
            ]),
            &mut mint,
        );

        assert_eq!(raised.len(), 2);
        assert_eq!(raised[0].kind.as_str(), "agent.socket.killed");
        assert_eq!(
            raised[1].kind.as_str(),
            "agent.socket.kill_refused",
            "a kill asked for and not carried out is the same question at an incident as one \
             that was, and a journal that records only the successes answers it wrongly"
        );
    }

    #[test]
    fn the_finding_says_who_asked_and_by_what_road_because_that_is_the_new_road() {
        let mut mint = mint();
        let raised = findings(
            &report(vec![Killed::done(
                "tcp|0.0.0.0:4444",
                30211,
                None,
                "SIGTERM sent",
            )]),
            &mut mint,
        );
        let said = format!("{:?}", raised[0].evidence);

        assert!(said.contains("the local console socket"), "{said}");
        assert!(said.contains("terminate"), "{said}");
        assert!(
            raised[0].title.contains("An operator"),
            "{}",
            raised[0].title
        );
    }

    #[test]
    fn two_kills_of_the_same_socket_are_one_finding_key_and_two_events() {
        let mut mint = mint();
        let once = findings(
            &report(vec![Killed::done("tcp|0.0.0.0:4444", 1, None, "sent")]),
            &mut mint,
        );
        let again = findings(
            &report(vec![Killed::done("tcp|0.0.0.0:4444", 2, None, "sent")]),
            &mut mint,
        );

        assert_eq!(once[0].finding_key, again[0].finding_key);
        assert_ne!(once[0].event_id, again[0].event_id);
    }
}
