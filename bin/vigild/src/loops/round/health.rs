use vigil_collect::Health;
use vigil_model::Finding;

use crate::helpers::{agent_finding, health, rfc3339};
use crate::types::Said;

use super::Round;

impl Round {
    pub(super) fn take_health(&mut self, announce: bool, said: &mut Said) {
        let mut raised: Vec<Finding> = Vec::new();

        for index in 0..self.watches.len() {
            raised.extend(self.health_of(index, said, announce));
        }

        self.raise(raised);
    }

    pub(super) fn take_health_of(&mut self, index: usize, said: &mut Said) {
        let raised = self.health_of(index, said, true);
        self.raise(raised);
    }

    fn health_of(&mut self, index: usize, said: &mut Said, announce: bool) -> Vec<Finding> {
        let state_of_it = self.watches[index].health();
        let name = self.watches[index].name();
        self.shared
            .with(|state| state.record_health(name, &state_of_it));

        let moved = said.health_moved(name, health::describe(&state_of_it));
        if !announce {
            return Vec::new();
        }

        match &state_of_it {
            Health::Ok => self.of_a_collector_that_is_well(name, said),
            Health::Degraded(why) => self.of_a_collector_that_is_not(name, why, false, said, moved),
            Health::Unavailable(why) => {
                self.of_a_collector_that_is_not(name, why, true, said, moved)
            }
        }
    }

    fn of_a_collector_that_is_well(&self, name: &'static str, said: &mut Said) -> Vec<Finding> {
        let Some(was) = said.closed(name) else {
            return Vec::new();
        };

        eprintln!("{} collector {name}: ok again", rfc3339::now());
        vec![agent_finding::collector_recovered(name, &was)]
    }

    fn of_a_collector_that_is_not(
        &self,
        name: &'static str,
        why: &str,
        fatal: bool,
        said: &mut Said,
        moved: Option<String>,
    ) -> Vec<Finding> {
        if moved.is_none() {
            return Vec::new();
        }

        match fatal {
            true => eprintln!("{} collector {name}: unavailable — {why}", rfc3339::now()),
            false => eprintln!("{} collector {name}: degraded — {why}", rfc3339::now()),
        }
        said.opened(
            name,
            match fatal {
                true => format!("unavailable — {why}"),
                false => format!("degraded — {why}"),
            },
        );
        vec![agent_finding::collector_degraded(name, why, fatal)]
    }

    pub(super) fn raise(&mut self, raised: Vec<Finding>) {
        if raised.is_empty() {
            return;
        }

        let fresh = self.remember(&raised);
        self.shared.with(|state| state.record_findings(&fresh));
        self.hand_over(&fresh);
    }
}
