use std::time::{Duration, Instant};

pub struct Due {
    name: &'static str,
    every_seconds: u32,
    next: Instant,
    skipped: u64,
}

impl Due {
    pub fn new(name: &'static str, every_seconds: u32, first_at: Instant) -> Self {
        Due {
            name,
            every_seconds: every_seconds.max(1),
            next: first_at,
            skipped: 0,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn every_seconds(&self) -> u32 {
        self.every_seconds
    }

    pub fn skipped(&self) -> u64 {
        self.skipped
    }

    pub fn next(&self) -> Instant {
        self.next
    }

    pub fn is_due(&self, now: Instant) -> bool {
        self.next <= now
    }

    pub fn waiting(&self, now: Instant) -> Duration {
        self.next.saturating_duration_since(now)
    }

    pub fn advance(&mut self, now: Instant) {
        let period = Duration::from_secs(u64::from(self.every_seconds));
        let mut slots: u64 = 0;
        while self.next <= now {
            self.next += period;
            slots += 1;
        }
        self.skipped += slots.saturating_sub(1);
    }

    pub fn restart(&mut self, now: Instant) {
        self.next = now + Duration::from_secs(u64::from(self.every_seconds));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(base: Instant, seconds: u64) -> Instant {
        base + Duration::from_secs(seconds)
    }

    #[test]
    fn a_missed_slot_is_skipped_and_counted_rather_than_run_twice() {
        let base = Instant::now();
        let mut due = Due::new("network", 30, base);

        due.advance(at(base, 95));

        assert_eq!(
            due.skipped, 3,
            "the slots at 30, 60 and 90 went by while the host slept; one reading answers, three are lost"
        );
        assert_eq!(due.next, at(base, 120));
        assert!(!due.is_due(at(base, 95)));
    }

    #[test]
    fn a_reading_taken_on_time_loses_no_slot() {
        let base = Instant::now();
        let mut due = Due::new("network", 30, base);

        due.advance(base);
        assert_eq!(due.skipped, 0);
        assert_eq!(due.next, at(base, 30));

        due.advance(at(base, 30));
        assert_eq!(due.skipped, 0);
        assert_eq!(due.next, at(base, 60));
    }

    #[test]
    fn a_reading_that_outlasts_its_own_period_does_not_start_the_next_one_behind() {
        let base = Instant::now();
        let mut due = Due::new("launches", 15, base);

        due.advance(at(base, 40));

        assert_eq!(
            due.next,
            at(base, 45),
            "the next slot is ahead, never behind"
        );
        assert_eq!(due.skipped, 2);
    }

    #[test]
    fn a_collector_that_is_not_due_is_left_where_it_was() {
        let base = Instant::now();
        let mut due = Due::new("users", 300, at(base, 100));

        due.advance(base);

        assert_eq!(due.next, at(base, 100));
        assert_eq!(due.skipped, 0);
        assert_eq!(due.waiting(base), Duration::from_secs(100));
    }

    #[test]
    fn a_reading_taken_out_of_turn_starts_the_period_again_and_counts_no_slot_as_lost() {
        let base = Instant::now();
        let mut due = Due::new("users", 300, at(base, 100));

        due.restart(at(base, 40));

        assert_eq!(
            due.next,
            at(base, 340),
            "the host was just read; reading it again sixty seconds later buys nothing"
        );
        assert_eq!(
            due.skipped, 0,
            "a slot that was not reached is not a slot that went by unread"
        );
    }

    #[test]
    fn a_period_of_zero_seconds_is_read_as_one_rather_than_spun_on() {
        let base = Instant::now();
        let mut due = Due::new("network", 0, base);

        assert_eq!(due.every_seconds(), 1);
        due.advance(base);
        assert_eq!(due.next, at(base, 1));
    }
}
