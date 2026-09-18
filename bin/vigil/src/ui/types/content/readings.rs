use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use vigil_model::Snapshot;

use super::refusal::Refusal;

pub enum Reading {
    Unknown,
    NotTakenYet,
    Refused(Refusal),
    Taken(Snapshot),
}

pub struct Readings {
    taken: BTreeMap<String, Reading>,
    generation: u64,
}

static NOT_ASKED: Reading = Reading::Unknown;

static GENERATIONS: AtomicU64 = AtomicU64::new(1);

fn next_generation() -> u64 {
    GENERATIONS.fetch_add(1, Ordering::Relaxed)
}

impl Default for Readings {
    fn default() -> Readings {
        Readings {
            taken: BTreeMap::new(),
            generation: next_generation(),
        }
    }
}

impl Readings {
    pub fn of(&self, collector: &str) -> &Reading {
        self.taken.get(collector).unwrap_or(&NOT_ASKED)
    }

    pub fn put(&mut self, collector: impl Into<String>, reading: Reading) {
        self.taken.insert(collector.into(), reading);
        self.generation = next_generation();
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_collector_nobody_has_asked_about_is_not_a_collector_that_read_nothing() {
        let mut readings = Readings::default();
        readings.put(
            "network",
            Reading::Taken(Snapshot::new("network", "2026-09-09T09:00:00.000Z")),
        );

        assert!(matches!(readings.of("users"), Reading::Unknown));
        assert!(
            matches!(readings.of("network"), Reading::Taken(_)),
            "and one that was asked and answered empty is an empty host, not a silence"
        );
    }
}
