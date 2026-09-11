use std::time::{Duration, Instant};

use super::Due;

pub struct Schedule {
    due: Vec<Due>,
}

impl Schedule {
    pub fn of(due: Vec<Due>) -> Self {
        Schedule { due }
    }

    pub fn name(&self, index: usize) -> Option<&'static str> {
        self.due.get(index).map(Due::name)
    }

    pub fn every_seconds(&self, index: usize) -> Option<u32> {
        self.due.get(index).map(Due::every_seconds)
    }

    pub fn skipped(&self, index: usize) -> u64 {
        self.due.get(index).map(Due::skipped).unwrap_or(0)
    }

    pub fn waiting(&self, index: usize, now: Instant) -> Duration {
        self.due
            .get(index)
            .map(|due| due.waiting(now))
            .unwrap_or_default()
    }

    pub fn periods(&self) -> Vec<(&'static str, u32)> {
        self.due
            .iter()
            .map(|due| (due.name(), due.every_seconds()))
            .collect()
    }

    pub fn due_now(&self, now: Instant) -> Option<usize> {
        self.due
            .iter()
            .enumerate()
            .filter(|(_, due)| due.is_due(now))
            .min_by_key(|(_, due)| due.next())
            .map(|(index, _)| index)
    }

    pub fn advance(&mut self, index: usize, now: Instant) {
        if let Some(due) = self.due.get_mut(index) {
            due.advance(now);
        }
    }

    pub fn rest(&self, now: Instant) -> Option<Duration> {
        self.due.iter().map(|due| due.waiting(now)).min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, seconds: u64) -> Instant {
        base + Duration::from_secs(seconds)
    }

    fn park(base: Instant) -> Schedule {
        Schedule::of(vec![
            Due::new("ports", 30, at(base, 4)),
            Due::new("launches", 15, at(base, 9)),
            Due::new("persistence", 300, at(base, 175)),
        ])
    }

    #[test]
    fn a_cheap_collector_does_not_wait_for_an_expensive_one() {
        let base = Instant::now();
        let mut schedule = park(base);

        let mut woken: Vec<&'static str> = Vec::new();
        for second in 0..=60 {
            let now = at(base, second);
            while let Some(index) = schedule.due_now(now) {
                woken.push(schedule.name(index).expect("a name"));
                schedule.advance(index, now);
            }
        }

        assert_eq!(
            woken,
            vec![
                "ports", "launches", "launches", "ports", "launches", "launches"
            ],
            "launches reads four times while persistence has not read once"
        );
    }

    #[test]
    fn the_schedule_hands_out_one_collector_at_a_time_and_never_a_set() {
        let base = Instant::now();
        let mut schedule = Schedule::of(vec![
            Due::new("ports", 30, base),
            Due::new("processes", 30, base),
        ]);

        let first = schedule.due_now(base).expect("one of them");
        schedule.advance(first, base);
        let second = schedule
            .due_now(base)
            .expect("then the other, one index at a time");
        schedule.advance(second, base);

        assert_ne!(first, second);
        assert_eq!(
            schedule.due_now(at(base, 1)),
            None,
            "both slots are taken; the next one is a period away"
        );
    }

    #[test]
    fn the_sleep_ends_at_the_earliest_due_time_and_not_a_fixed_interval() {
        let base = Instant::now();
        let mut schedule = park(base);

        assert_eq!(
            schedule.rest(base),
            Some(Duration::from_secs(4)),
            "the wait is until the first collector, not a period of its own"
        );

        let ports = schedule
            .due_now(at(base, 4))
            .expect("ports at four seconds");
        schedule.advance(ports, at(base, 4));

        assert_eq!(
            schedule.rest(at(base, 4)),
            Some(Duration::from_secs(5)),
            "launches at nine seconds is next, not ports at thirty-four"
        );
        assert_eq!(schedule.rest(at(base, 200)), Some(Duration::ZERO));
    }

    #[test]
    fn a_host_watching_nothing_has_nothing_to_wait_for() {
        let schedule = Schedule::of(Vec::new());

        assert!(schedule.periods().is_empty());
        assert_eq!(schedule.rest(Instant::now()), None);
        assert_eq!(schedule.due_now(Instant::now()), None);
    }

    #[test]
    fn the_most_overdue_collector_is_read_first() {
        let base = Instant::now();
        let mut schedule = Schedule::of(vec![
            Due::new("ports", 30, at(base, 20)),
            Due::new("launches", 15, at(base, 10)),
        ]);

        let index = schedule.due_now(at(base, 25)).expect("both are due");
        assert_eq!(schedule.name(index), Some("launches"));

        schedule.advance(index, at(base, 25));
        assert_eq!(
            schedule.skipped(index),
            1,
            "the slot at 10 and the slot at 25 both came due; one reading answers both"
        );
        assert_eq!(
            schedule.waiting(index, at(base, 25)),
            Duration::from_secs(15)
        );
    }
}
