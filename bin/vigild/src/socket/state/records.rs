use vigil_collect::Health;
use vigil_model::{
    AgentBudget, BufferStatus, CollectorState, CollectorStatus, Finding, Kind, Rfc3339, Silence,
    StoreStatus,
};

use super::State;
use crate::types::Reading;

impl State {
    pub fn record_reading(&mut self, reading: Reading) {
        if let Some(status) = self.collector_mut(reading.collector) {
            status.readings += 1;
            status.last_run_at = Some(reading.at);
            status.duration_ms = Some(reading.duration_ms);
            status.items = reading.snapshot.items.len();
            status.every_seconds = Some(reading.every_seconds);
            status.next_run_at = Some(reading.next_run_at);
            status.skipped = reading.skipped;
            status.baseline = true;
        }
        self.snapshots
            .insert(reading.collector.to_string(), reading.snapshot);
    }

    pub fn record_budget(&mut self, duty_percent: Option<f64>, resident_kb: Option<u64>) {
        self.footprint.budget = AgentBudget {
            duty_percent,
            resident_kb,
        };
    }

    pub fn record_store(&mut self, store: StoreStatus) {
        self.footprint.store = Some(store);
    }

    pub fn record_buffers(&mut self, buffers: Vec<BufferStatus>) {
        self.footprint.buffers = Some(buffers);
    }

    pub fn record_health(&mut self, collector: &str, health: &Health) {
        let Some(status) = self.collector_mut(collector) else {
            return;
        };
        if status.state == CollectorState::Off {
            return;
        }

        status.state = match health {
            Health::Ok => CollectorState::Ok,
            Health::Degraded(_) => CollectorState::Degraded,
            Health::Unavailable(_) => CollectorState::Unavailable,
        };
        status.reason = match health {
            Health::Ok => None,
            Health::Degraded(why) | Health::Unavailable(why) => Some(why.clone()),
        };
    }

    pub fn record_failure(&mut self, collector: &str, at: Rfc3339, error: &str) {
        if let Some(status) = self.collector_mut(collector) {
            status.failures += 1;
            status.last_run_at = Some(at);
            status.last_error = Some(error.to_string());
            if status.state == CollectorState::Ok {
                status.state = CollectorState::Degraded;
                status.reason = Some(error.to_string());
            }
        }
    }

    pub fn record_findings(&mut self, findings: &[Finding]) {
        for finding in findings {
            if let Kind::Known(kind) = &finding.kind
                && let Some(ended) = kind.resolves()
            {
                self.findings.resolve(&finding.finding_key, ended.as_str());
            }
            self.findings.push(finding.clone());
        }
    }

    pub fn recall_findings(&mut self, history: Vec<Finding>, held: u64) {
        self.findings.recall(history, held);
    }

    pub fn record_policy(&mut self, suppressions: Vec<String>, suppressed: u64) {
        self.silence = Silence {
            suppressed,
            suppressions,
        };
    }

    pub fn record_delivery(&mut self, reporter: &str, at: Rfc3339, error: Option<String>) {
        let Some(status) = self.reporters.iter_mut().find(|it| it.name == reporter) else {
            return;
        };
        match error {
            None => {
                status.deliveries += 1;
                status.last_sent_at = Some(at);
            }
            Some(error) => {
                status.failures += 1;
                status.last_error = Some(error);
            }
        }
    }

    fn collector_mut(&mut self, name: &str) -> Option<&mut CollectorStatus> {
        self.collectors.iter_mut().find(|it| it.name == name)
    }
}
