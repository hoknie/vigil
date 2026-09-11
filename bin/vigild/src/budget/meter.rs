use super::resident;
use super::{CollectorCost, Crossing, Gate};
use crate::types::Reading;

pub const CPU: &str = "cpu";
pub const RESIDENT: &str = "resident";
pub const CEILING_PERCENT: f64 = 1.0;

#[derive(Default)]
pub struct Meter {
    costs: Vec<CollectorCost>,
    cpu: Gate,
    resident: Gate,
}

impl Meter {
    pub fn record(&mut self, reading: &Reading) {
        match self
            .costs
            .iter_mut()
            .find(|cost| cost.name() == reading.collector)
        {
            Some(known) => known.record(reading),
            None => self.costs.push(CollectorCost::of(reading)),
        }
    }

    pub fn duty_percent(&self) -> Option<f64> {
        if self.costs.is_empty() {
            return None;
        }
        Some(self.costs.iter().map(CollectorCost::duty_percent).sum())
    }

    pub fn check_cpu(&mut self) -> Option<Crossing> {
        let duty = self.duty_percent()?;
        self.cpu.judge(duty > CEILING_PERCENT)
    }

    pub fn check_resident(&mut self, kilobytes: Option<u64>) -> Option<Crossing> {
        let measured = kilobytes?;
        self.resident.judge(measured > resident::CEILING_KB)
    }

    pub fn costs(&self) -> Vec<String> {
        self.costs
            .iter()
            .map(|cost| {
                let missed = match cost.skipped() {
                    0 => String::new(),
                    slots => format!(" · {slots} slot(s) missed"),
                };
                format!(
                    "{} · {:.2} % of one core · every {} s · {:.1} ms per reading{missed}",
                    cost.name(),
                    cost.duty_percent(),
                    cost.every_seconds(),
                    cost.average_ms(),
                )
            })
            .collect()
    }

    pub fn schedule_line(&self) -> String {
        let duty = self.duty_percent().unwrap_or(0.0);
        let stretch = match duty > CEILING_PERCENT {
            true => duty / CEILING_PERCENT,
            false => 1.0,
        };

        let mut line = String::from("schedule:\n");
        for cost in &self.costs {
            let wanted = (f64::from(cost.every_seconds()) * stretch).ceil() as u32;
            line.push_str(&format!(
                "  {}: {}\n",
                cost.name(),
                wanted.max(cost.every_seconds() + 1)
            ));
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Snapshot;

    use super::*;

    fn reading(collector: &'static str, every_seconds: u32, duration_ms: u64) -> Reading {
        Reading {
            collector,
            at: "2026-09-10T12:00:00.000Z".into(),
            duration_ms,
            every_seconds,
            next_run_at: "2026-09-10T12:00:30.000Z".into(),
            skipped: 0,
            snapshot: Snapshot::new(collector, "2026-09-10T12:00:00.000Z"),
        }
    }

    fn cheap() -> Meter {
        let mut meter = Meter::default();
        meter.record(&reading("ports", 30, 4));
        meter.record(&reading("processes", 30, 1));
        meter.record(&reading("launches", 15, 1));
        meter
    }

    #[test]
    fn an_agent_that_has_read_nothing_yet_makes_no_claim_about_its_cost() {
        let mut meter = Meter::default();

        assert_eq!(meter.duty_percent(), None);
        assert_eq!(meter.check_cpu(), None);
    }

    #[test]
    fn the_shipped_schedule_costs_a_fiftieth_of_what_it_is_allowed() {
        let duty = cheap().duty_percent().expect("three readings");

        assert!(duty < CEILING_PERCENT / 10.0, "{duty} % of one core");
    }

    #[test]
    fn the_cost_of_the_agent_is_the_sum_of_its_collectors_and_not_the_worst_of_them() {
        let mut meter = Meter::default();
        meter.record(&reading("ports", 30, 150));
        meter.record(&reading("processes", 30, 150));

        let duty = meter.duty_percent().expect("two readings");

        assert!((duty - 1.0).abs() < 0.000_1, "{duty}");
        assert_eq!(meter.check_cpu(), None, "at the ceiling is not over it");
    }

    #[test]
    fn a_platform_that_cannot_measure_its_own_memory_says_so_instead_of_reporting_zero() {
        let mut meter = cheap();

        assert_eq!(
            meter.check_resident(None),
            None,
            "not measured is not measured as fine"
        );
        assert_eq!(meter.check_resident(Some(12_000)), None);
        assert_eq!(
            meter.check_resident(Some(resident::CEILING_KB + 1)),
            Some(Crossing::Exceeded)
        );
    }

    #[test]
    fn going_over_the_ceiling_is_said_and_the_way_back_is_a_period_a_person_can_copy() {
        let mut meter = Meter::default();
        meter.record(&reading("ports", 30, 600));

        assert_eq!(meter.check_cpu(), Some(Crossing::Exceeded));
        assert_eq!(
            meter.schedule_line(),
            "schedule:\n  ports: 60\n",
            "600 ms every 30 s is 2 % of a core; twice the period is the way back under 1 %"
        );
    }

    #[test]
    fn a_schedule_under_the_ceiling_is_still_a_schedule_that_reads() {
        let line = cheap().schedule_line();

        assert!(line.starts_with("schedule:\n"), "{line}");
        for name in ["ports", "processes", "launches"] {
            assert!(line.contains(&format!("  {name}: ")), "{line}");
        }
    }
}
