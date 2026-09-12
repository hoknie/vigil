use std::time::Instant;

use vigil_model::Finding;
use vigil_store::Store;

use crate::helpers::rfc3339;
use crate::types::{Reading, Said};

use super::Round;
use crate::loops::Tick;

impl Round {
    pub(super) fn read(&mut self, index: usize, said: &mut Said) {
        let name = self.watches[index].name();
        debug_assert_eq!(
            self.schedule.name(index),
            Some(name),
            "the schedule and the watches are one list read by one index"
        );

        let started = Instant::now();
        let outcome = self.watches[index].tick();
        let duration_ms = started.elapsed().as_millis() as u64;

        match outcome {
            Ok(tick) => {
                self.keep(&tick, name);
                let fresh = self.remember(&tick.findings);
                self.publish(self.taken(index, duration_ms, &tick), &fresh);
                self.report(name, &tick, &fresh);

                let had_failed = said.reading_again(name);
                if had_failed {
                    eprintln!("{} collector reading again: {name}", rfc3339::now());
                }
                if had_failed || tick.changes > 0 {
                    self.take_health_of(index, said);
                }
            }
            Err(error) => {
                self.shared
                    .with(|state| state.record_failure(name, rfc3339::now(), &error));
                if said.failing(name, &error) {
                    eprintln!("{} collector failed: {error}", rfc3339::now());
                }
            }
        }
    }

    fn taken(&self, index: usize, duration_ms: u64, tick: &Tick) -> Reading {
        Reading {
            collector: self.watches[index].name(),
            at: rfc3339::now(),
            duration_ms,
            every_seconds: self.schedule.every_seconds(index).unwrap_or_default(),
            next_run_at: rfc3339::ahead(self.schedule.waiting(index, Instant::now())),
            skipped: self.schedule.skipped(index),
            snapshot: tick.snapshot.clone(),
        }
    }

    fn keep(&self, tick: &Tick, name: &str) {
        if let Err(error) = self.store.set_baseline(&tick.snapshot) {
            eprintln!("{} baseline for {name} not kept: {error}", rfc3339::now());
        }
    }

    fn publish(&mut self, reading: Reading, fresh: &[Finding]) {
        self.meter.record(&reading);
        let suppressions = self.policy.describe_suppressions();
        let suppressed = self.policy.suppressed();
        self.shared.with(|state| {
            state.record_reading(reading);
            state.record_findings(fresh);
            state.record_policy(suppressions, suppressed);
        });
    }

    fn report(&mut self, collector: &str, tick: &Tick, fresh: &[Finding]) {
        if tick.baseline {
            eprintln!(
                "{} baseline: {} item(s) in {collector}",
                rfc3339::now(),
                tick.snapshot.items.len()
            );
            return;
        }

        if tick.changes > 0 {
            eprintln!(
                "{} {collector}: {} change(s), {} finding(s), {} new",
                rfc3339::now(),
                tick.changes,
                tick.findings.len(),
                fresh.len()
            );
        }
        for finding in fresh {
            eprintln!(
                "  [{}] {} — {}",
                finding.severity, finding.kind, finding.title
            );
        }

        self.hand_over(fresh);
    }
}
