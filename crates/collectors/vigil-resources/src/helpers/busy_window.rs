use std::collections::VecDeque;

pub const WINDOW_SECONDS: u64 = 900;

const STEP: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sample {
    pub at_seconds: u64,
    pub total: u64,
    pub busy: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Busy {
    pub percent_step: Option<u64>,
    pub window_seconds: u64,
}

#[derive(Debug, Default)]
pub struct BusyWindow {
    samples: VecDeque<Sample>,
}

impl BusyWindow {
    pub fn push(&mut self, sample: Sample) -> Busy {
        if let Some(last) = self.samples.back()
            && (sample.total < last.total
                || sample.busy < last.busy
                || sample.at_seconds < last.at_seconds)
        {
            self.samples.clear();
        }
        self.samples.push_back(sample);

        let anchor = self
            .samples
            .iter()
            .rposition(|old| sample.at_seconds - old.at_seconds >= WINDOW_SECONDS);
        let Some(anchor) = anchor else {
            return Busy {
                percent_step: None,
                window_seconds: WINDOW_SECONDS,
            };
        };
        self.samples.drain(..anchor);

        let old = self.samples[0];
        let total = sample.total - old.total;
        let busy = sample.busy - old.busy;
        let percent_step = match total {
            0 => None,
            total => Some(busy.min(total) * 100 / total / STEP * STEP),
        };

        Busy {
            percent_step,
            window_seconds: sample.at_seconds - old.at_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICKS_A_MINUTE: u64 = 6_000;

    fn minutes(window: &mut BusyWindow, start: &mut Sample, percents: &[u64]) -> Busy {
        let mut last = window.push(*start);
        for percent in percents {
            start.at_seconds += 60;
            start.total += TICKS_A_MINUTE;
            start.busy += TICKS_A_MINUTE * percent / 100;
            last = window.push(*start);
        }
        last
    }

    fn origin() -> Sample {
        Sample {
            at_seconds: 1_000,
            total: 1_000_000,
            busy: 200_000,
        }
    }

    #[test]
    fn busy_is_not_known_until_fifteen_minutes_of_readings_are_held() {
        let mut window = BusyWindow::default();

        let after_five = minutes(&mut window, &mut origin(), &[100; 5]);

        assert_eq!(
            after_five.percent_step, None,
            "five minutes of readings is not fifteen, and an unknown is not a zero"
        );
    }

    #[test]
    fn a_one_minute_spike_on_a_quiet_host_stays_in_the_lowest_steps() {
        let mut window = BusyWindow::default();
        let mut quiet = [5; 15];
        quiet[14] = 100;

        let busy = minutes(&mut window, &mut origin(), &quiet);

        assert_eq!(busy.percent_step, Some(10));
        assert_eq!(busy.window_seconds, WINDOW_SECONDS);
    }

    #[test]
    fn a_host_busy_for_longer_than_the_window_reads_as_its_step() {
        let mut window = BusyWindow::default();

        let busy = minutes(&mut window, &mut origin(), &[97; 30]);

        assert_eq!(busy.percent_step, Some(90));
        assert_eq!(busy.window_seconds, WINDOW_SECONDS);
    }

    #[test]
    fn counters_that_went_backwards_start_the_window_again() {
        let mut window = BusyWindow::default();
        minutes(&mut window, &mut origin(), &[50; 20]);

        let rebooted = window.push(Sample {
            at_seconds: 10_000,
            total: 10,
            busy: 5,
        });

        assert_eq!(rebooted.percent_step, None);
    }

    #[test]
    fn a_period_longer_than_the_window_averages_between_two_readings() {
        let mut window = BusyWindow::default();
        window.push(origin());

        let busy = window.push(Sample {
            at_seconds: origin().at_seconds + 3_600,
            total: origin().total + 360_000,
            busy: origin().busy + 180_000,
        });

        assert_eq!(busy.percent_step, Some(50));
        assert_eq!(busy.window_seconds, 3_600);
    }
}
