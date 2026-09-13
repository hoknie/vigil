use vigil_model::{
    AgentStatus, CollectorRefusal, CollectorState, CollectorStatus, Finding, FindingsSummary, Host,
    Snapshot,
};

use super::State;
use super::limitations::limitations;

const WITHOUT_A_WORD: &str = "the collector reports trouble and named no cause";

impl State {
    pub fn host(&self) -> &Host {
        &self.startup.host
    }

    pub fn agent(&self) -> AgentStatus {
        AgentStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            started_at: self.startup.started_at.clone(),
            configuration_path: Some(self.startup.configuration_path.clone()),
            interval_seconds: self.startup.interval_seconds,
            collectors: self.collectors.clone(),
            reporters: self.reporters.clone(),
            findings: FindingsSummary {
                retained: self.findings.len(),
                capacity: self.findings.capacity(),
                total: self.findings.total(),
                dropped: self.findings.dropped(),
                by_severity: self.findings.by_severity(),
            },
            silence: self.silence.clone(),
            budget: self.footprint.budget.clone(),
            store: self.footprint.store.clone(),
            buffers: self.footprint.buffers.clone(),
            limitations: limitations(
                self.footprint.store.as_ref(),
                self.footprint.buffers.as_deref(),
            ),
        }
    }

    pub fn collector_names(&self) -> Vec<String> {
        self.collectors
            .iter()
            .filter(|collector| collector.state != CollectorState::Off)
            .map(|collector| collector.name.clone())
            .collect()
    }

    pub fn collector(&self, name: &str) -> Option<CollectorStatus> {
        self.collectors
            .iter()
            .find(|collector| collector.name == name)
            .cloned()
    }

    pub fn knows_collector(&self, name: &str) -> bool {
        self.collectors
            .iter()
            .any(|it| it.name == name && it.state != CollectorState::Off)
    }

    pub fn snapshot(&self, collector: &str) -> Option<&Snapshot> {
        self.snapshots.get(collector)
    }

    pub fn refusal(&self, collector: &str) -> Option<CollectorRefusal> {
        let status = self.collectors.iter().find(|it| it.name == collector)?;
        if !status.state.is_trouble() {
            return None;
        }
        let reason = status
            .reason
            .clone()
            .or_else(|| status.last_error.clone())
            .unwrap_or_else(|| WITHOUT_A_WORD.to_string());
        Some(CollectorRefusal::new(status.state.clone(), reason))
    }

    pub fn latest_findings(&self, limit: Option<usize>) -> Vec<Finding> {
        self.findings.latest(limit)
    }

    pub fn findings_dropped(&self) -> u64 {
        self.findings.dropped()
    }

    pub fn findings_capacity(&self) -> usize {
        self.findings.capacity()
    }
}
