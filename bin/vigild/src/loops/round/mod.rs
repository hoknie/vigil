mod budget;
mod health;
mod kept;
mod memory;
mod reading;

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use vigil_store::FileStore;

use crate::budget::Meter;
use crate::helpers::health as health_words;
use crate::socket::Shared;
use crate::types::{Delivery, Due, Policy, Schedule};

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
}

impl Round {
    pub fn run(mut self) -> ! {
        let mut failing: BTreeMap<&'static str, String> = BTreeMap::new();
        let mut announced: BTreeMap<&'static str, String> = self
            .watches
            .iter()
            .map(|watch| (watch.name(), health_words::describe(&watch.health())))
            .collect();
        let mut health = Due::new(HEALTH, HEALTH_EVERY_SECONDS, Instant::now());
        let mut announce = false;

        for index in 0..self.watches.len() {
            self.read(index, &mut failing);
        }
        self.take_store();

        loop {
            let now = Instant::now();
            if health.is_due(now) {
                self.take_health(announce, &mut announced);
                self.take_budget();
                self.take_store();
                health.advance(Instant::now());
                announce = true;
            }

            match self.schedule.due_now(Instant::now()) {
                Some(index) => {
                    self.schedule.advance(index, Instant::now());
                    self.read(index, &mut failing);
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
