mod budget;
mod health;
mod kept;
mod memory;
mod outgoing;
mod reading;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use vigil_model::Finding;
use vigil_store::FileStore;

use crate::budget::Meter;
use crate::helpers::health as health_words;
use crate::socket::Shared;
use crate::types::{Delivery, Due, Policy, Said, Schedule};

use super::Watch;

const HEALTH_EVERY_SECONDS: u32 = 300;
const HEALTH: &str = "health";
const NEVER_SPIN: Duration = Duration::from_millis(10);

pub struct Round {
    pub watches: Vec<Watch>,
    pub store: FileStore,
    pub policy: Policy,
    pub delivery: Delivery,
    pub shared: Shared,
    pub schedule: Schedule,
    pub meter: Meter,
    pub opening: Vec<Finding>,
}

impl Round {
    pub fn run(mut self) -> ! {
        let mut said = Said::about(
            self.watches
                .iter()
                .map(|watch| (watch.name(), health_words::describe(&watch.health())))
                .collect::<Vec<_>>(),
        );
        let mut health = Due::new(HEALTH, HEALTH_EVERY_SECONDS, Instant::now());
        let mut announce = false;

        let opening = std::mem::take(&mut self.opening);
        let fresh = self.remember(&opening);
        self.shared.with(|state| state.record_findings(&fresh));
        self.hand_over(&fresh);

        for index in 0..self.watches.len() {
            self.read(index, &mut said);
        }
        self.take_store();

        loop {
            let now = Instant::now();
            if health.is_due(now) {
                self.take_health(announce, &mut said);
                self.take_budget();
                self.take_store();
                self.take_buffers();
                health.advance(Instant::now());
                announce = true;
            }

            match self.schedule.due_now(Instant::now()) {
                Some(index) => {
                    self.schedule.advance(index, Instant::now());
                    self.read(index, &mut said);
                }
                None => self.rest(&health),
            }
        }
    }

    fn rest(&self, health: &Due) {
        let now = Instant::now();
        let until = match self.schedule.rest(now) {
            Some(soonest) => soonest.min(health.waiting(now)),
            None => health.waiting(now),
        };

        std::thread::sleep(until.max(NEVER_SPIN));
    }
}
