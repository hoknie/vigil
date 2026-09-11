use std::collections::BTreeMap;

use vigil_collect::Health;
use vigil_model::{CollectorState, CollectorStatus, ReporterStatus, Silence};

use crate::socket::{Ring, switched_off_reason};
use crate::types::Startup;

use super::State;
use super::footprint::Footprint;

const RETAINED_FINDINGS: usize = 500;

impl State {
    pub fn new(
        startup: Startup,
        collectors: &[(&str, Health)],
        reporters: &[String],
        switched_off: &[String],
    ) -> Self {
        let watching: Vec<CollectorStatus> = collectors
            .iter()
            .map(|(name, health)| CollectorStatus {
                name: (*name).to_string(),
                state: match health {
                    Health::Ok => CollectorState::Ok,
                    Health::Degraded(_) => CollectorState::Degraded,
                    Health::Unavailable(_) => CollectorState::Unavailable,
                },
                reason: match health {
                    Health::Ok => None,
                    Health::Degraded(detail) | Health::Unavailable(detail) => Some(detail.clone()),
                },
                last_run_at: None,
                duration_ms: None,
                items: 0,
                readings: 0,
                every_seconds: startup.periods.get(*name).copied(),
                next_run_at: None,
                skipped: 0,
                failures: 0,
                last_error: None,
                baseline: false,
            })
            .collect();
        let told_not_to = switched_off.iter().map(|name| CollectorStatus {
            name: name.clone(),
            state: CollectorState::Off,
            reason: Some(switched_off_reason(name)),
            last_run_at: None,
            duration_ms: None,
            items: 0,
            readings: 0,
            every_seconds: None,
            next_run_at: None,
            skipped: 0,
            failures: 0,
            last_error: None,
            baseline: false,
        });

        State {
            startup,
            collectors: watching.into_iter().chain(told_not_to).collect(),
            reporters: reporters
                .iter()
                .map(|name| ReporterStatus {
                    name: name.clone(),
                    deliveries: 0,
                    failures: 0,
                    last_sent_at: None,
                    last_error: None,
                })
                .collect(),
            snapshots: BTreeMap::new(),
            findings: Ring::new(RETAINED_FINDINGS),
            silence: Silence::default(),
            footprint: Footprint::default(),
        }
    }
}
