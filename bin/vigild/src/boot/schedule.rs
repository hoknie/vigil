use std::time::{Duration, Instant};

use crate::Config;
use crate::helpers::phase;
use crate::loops::Watch;
use crate::types::{Due, Schedule};

pub fn of(config: &Config, watches: &[Watch], host_id: &str) -> Schedule {
    let start = Instant::now();

    Schedule::of(
        watches
            .iter()
            .map(|watch| {
                let name = watch.name();
                let every_seconds = config.every_seconds(name).max(1);
                let phase = phase::seconds(host_id, name, every_seconds);
                Due::new(
                    name,
                    every_seconds,
                    start + Duration::from_secs(u64::from(phase)),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use vigil_collect::{CollectError, Collector, Health};
    use vigil_model::Snapshot;
    use vigil_rules::RuleSet;

    use super::*;

    struct Named(&'static str);

    impl Collector for Named {
        fn name(&self) -> &'static str {
            self.0
        }
        fn available(&self) -> Health {
            Health::Ok
        }
        fn collect(&self) -> Result<Snapshot, CollectError> {
            Err(CollectError::Absent("a test collector".into()))
        }
    }

    fn watches() -> Vec<Watch> {
        ["ports", "users", "launches"]
            .into_iter()
            .map(|name| Watch::new(Box::new(Named(name)), RuleSet::of(Vec::new())))
            .collect()
    }

    #[test]
    fn the_schedule_is_the_list_of_watches_in_the_order_the_watches_are_in() {
        let schedule = of(&Config::default(), &watches(), "1c9d8e7b4a5c6d0e");

        assert_eq!(
            schedule.periods(),
            vec![("ports", 30), ("users", 300), ("launches", 15)],
            "the daemon reads both lists by one index"
        );
    }

    #[test]
    fn a_period_named_in_the_configuration_is_the_period_the_collector_gets() {
        let mut config = Config {
            interval_seconds: Some(10),
            ..Config::default()
        };
        config.schedule.insert("users".into(), 600);

        let schedule = of(&config, &watches(), "1c9d8e7b4a5c6d0e");

        assert_eq!(
            schedule.periods(),
            vec![("ports", 10), ("users", 600), ("launches", 10)]
        );
    }

    #[test]
    fn two_collectors_of_one_period_are_not_first_read_in_the_same_second() {
        let config = Config {
            interval_seconds: Some(30),
            ..Config::default()
        };

        let schedule = of(&config, &watches(), "1c9d8e7b4a5c6d0e");
        let now = Instant::now();

        let waits: Vec<u64> = (0..schedule.periods().len())
            .map(|index| schedule.waiting(index, now).as_secs())
            .collect();
        let mut alone = waits.clone();
        alone.sort_unstable();
        alone.dedup();

        assert_eq!(alone.len(), waits.len(), "{waits:?} share a second");
    }
}
