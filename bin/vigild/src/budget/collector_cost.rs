use crate::types::Reading;

const REMEMBERED_READINGS: usize = 8;

pub struct CollectorCost {
    name: String,
    every_seconds: u32,
    skipped: u64,
    recent_ms: Vec<u64>,
}

impl CollectorCost {
    pub fn of(reading: &Reading) -> Self {
        let mut cost = CollectorCost {
            name: reading.collector.to_string(),
            every_seconds: reading.every_seconds.max(1),
            skipped: 0,
            recent_ms: Vec::new(),
        };
        cost.record(reading);
        cost
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn every_seconds(&self) -> u32 {
        self.every_seconds
    }

    pub fn skipped(&self) -> u64 {
        self.skipped
    }

    pub fn record(&mut self, reading: &Reading) {
        self.every_seconds = reading.every_seconds.max(1);
        self.skipped = reading.skipped;
        if self.recent_ms.len() == REMEMBERED_READINGS {
            self.recent_ms.remove(0);
        }
        self.recent_ms.push(reading.duration_ms);
    }

    pub fn average_ms(&self) -> f64 {
        if self.recent_ms.is_empty() {
            return 0.0;
        }
        self.recent_ms.iter().sum::<u64>() as f64 / self.recent_ms.len() as f64
    }

    pub fn duty_percent(&self) -> f64 {
        self.average_ms() / (f64::from(self.every_seconds) * 1_000.0) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Snapshot;

    use super::*;

    fn reading(duration_ms: u64) -> Reading {
        Reading {
            collector: "network",
            at: "2026-09-10T12:00:00.000Z".into(),
            duration_ms,
            every_seconds: 30,
            next_run_at: "2026-09-10T12:00:30.000Z".into(),
            skipped: 0,
            snapshot: Snapshot::new("network", "2026-09-10T12:00:00.000Z"),
        }
    }

    #[test]
    fn a_single_slow_reading_is_not_a_broken_budget() {
        let mut cost = CollectorCost::of(&reading(4));
        for _ in 0..6 {
            cost.record(&reading(4));
        }
        cost.record(&reading(400));

        assert!(
            cost.duty_percent() < 0.25,
            "one slow second is the noise of a host, not a permanent cost: {} %",
            cost.duty_percent()
        );
    }

    #[test]
    fn the_share_of_a_core_is_the_reading_against_its_own_period() {
        let mut cost = CollectorCost::of(&reading(300));
        cost.record(&reading(300));

        assert!(
            (cost.duty_percent() - 1.0).abs() < 0.000_1,
            "300 ms every 30 s is one per cent of one core, got {}",
            cost.duty_percent()
        );
    }

    #[test]
    fn only_the_last_eight_readings_are_remembered() {
        let mut cost = CollectorCost::of(&reading(1_000));
        for _ in 0..8 {
            cost.record(&reading(0));
        }

        assert_eq!(
            cost.average_ms(),
            0.0,
            "a cost that has been paid off stays paid off"
        );
    }
}
