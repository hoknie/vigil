use vigil_collect::Health;
use vigil_model::Finding;

use crate::helpers::{agent_finding, health, rfc3339};
use crate::types::Said;

use super::Round;

impl Round {
    pub(super) fn take_health(&mut self, announce: bool, said: &mut Said) {
        let mut raised: Vec<Finding> = Vec::new();

        for index in 0..self.watches.len() {
            let state_of_it = self.watches[index].health();
            let name = self.watches[index].name();
            self.shared
                .with(|state| state.record_health(name, &state_of_it));

            let moved = said.health_moved(name, health::describe(&state_of_it));
            let Some(before) = moved.filter(|_| announce) else {
                continue;
            };

            self.announce(name, &state_of_it);
            raised.extend(of_health(name, &state_of_it, &before, said));
        }

        self.raise(raised);
    }

    pub(super) fn take_health_of(&mut self, index: usize, said: &mut Said) {
        let state_of_it = self.watches[index].health();
        let name = self.watches[index].name();
        self.shared
            .with(|state| state.record_health(name, &state_of_it));

        let Some(before) = said.health_moved(name, health::describe(&state_of_it)) else {
            return;
        };

        self.announce(name, &state_of_it);
        let raised = of_health(name, &state_of_it, &before, said);
        self.raise(raised);
    }

    fn raise(&mut self, raised: Vec<Finding>) {
        if raised.is_empty() {
            return;
        }

        let fresh = self.remember(&raised);
        self.shared.with(|state| state.record_findings(&fresh));
        self.hand_over(&fresh);
    }

    fn announce(&self, name: &'static str, state_of_it: &Health) {
        match state_of_it {
            Health::Ok => eprintln!("{} collector {name}: ok again", rfc3339::now()),
            Health::Degraded(why) => {
                eprintln!("{} collector {name}: degraded — {why}", rfc3339::now())
            }
            Health::Unavailable(why) => {
                eprintln!("{} collector {name}: unavailable — {why}", rfc3339::now())
            }
        }
    }
}

fn of_health(
    name: &'static str,
    state_of_it: &Health,
    before: &str,
    said: &mut Said,
) -> Vec<Finding> {
    match state_of_it {
        Health::Ok => match said.closed(name) {
            true => vec![agent_finding::collector_recovered(
                name,
                &health::in_words(before),
            )],
            false => Vec::new(),
        },
        Health::Degraded(why) => {
            said.opened(name);
            vec![agent_finding::collector_degraded(name, why, false)]
        }
        Health::Unavailable(why) => {
            said.opened(name);
            vec![agent_finding::collector_degraded(name, why, true)]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(findings: &[Finding]) -> Vec<String> {
        findings
            .iter()
            .map(|finding| finding.kind.as_str().to_string())
            .collect()
    }

    #[test]
    fn a_collector_that_reads_again_closes_the_finding_raised_when_it_stopped() {
        let mut said = Said::default();
        let went = of_health(
            "launches",
            &Health::Degraded("auditd has brought nothing".into()),
            "ok",
            &mut said,
        );

        let came_back = of_health(
            "launches",
            &Health::Ok,
            "degraded:auditd has brought nothing",
            &mut said,
        );

        assert_eq!(kinds(&went), vec!["agent.collector.degraded".to_string()]);
        assert_eq!(
            kinds(&came_back),
            vec!["agent.collector.recovered".to_string()]
        );
        assert_eq!(
            went[0].finding_key, came_back[0].finding_key,
            "one object, two statements: otherwise the first one stands for the life of the host"
        );
        assert!(
            came_back[0]
                .evidence
                .iter()
                .any(|evidence| evidence.value.contains("auditd has brought nothing")),
            "and what it was carries into what closes it: {:?}",
            came_back[0].evidence
        );
    }

    #[test]
    fn a_collector_nobody_complained_about_raises_nothing_by_being_well() {
        let mut said = Said::default();

        let fine = of_health("ports", &Health::Ok, "ok", &mut said);

        assert!(
            fine.is_empty(),
            "a finding that closes nothing is a finding an operator has to read for no reason"
        );
    }

    #[test]
    fn a_collector_that_came_back_twice_is_reported_once() {
        let mut said = Said::default();
        of_health(
            "launches",
            &Health::Unavailable("auditd is not running".into()),
            "ok",
            &mut said,
        );

        let first = of_health("launches", &Health::Ok, "unavailable:x", &mut said);
        let second = of_health("launches", &Health::Ok, "ok", &mut said);

        assert_eq!(first.len(), 1);
        assert!(
            second.is_empty(),
            "the pair closes a finding that is open, and after the first one there is none"
        );
    }

    #[test]
    fn a_collector_the_last_run_complained_about_is_still_spoken_about_after_a_restart() {
        let mut said = Said::about([(
            "launches",
            "degraded:auditd has brought nothing".to_string(),
        )]);

        let came_back = of_health("launches", &Health::Ok, "degraded:x", &mut said);

        assert_eq!(
            kinds(&came_back),
            vec!["agent.collector.recovered".to_string()],
            "the finding the greeting raised about a collector that was unwell at startup has \
             to be closable without waiting for it to break a second time"
        );
    }
}
