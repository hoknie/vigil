use serde::{Deserialize, Serialize};

use crate::{AgentBudget, CollectorStatus, FindingsSummary, ReporterStatus, Rfc3339, StoreStatus};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Silence {
    #[serde(default)]
    pub suppressed: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppressions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub version: String,
    pub started_at: Rfc3339,
    pub interval_seconds: u32,
    pub collectors: Vec<CollectorStatus>,
    pub reporters: Vec<ReporterStatus>,
    pub findings: FindingsSummary,
    #[serde(default)]
    pub silence: Silence,
    #[serde(default)]
    pub budget: AgentBudget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<StoreStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub limitations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an_agent_that_says_nothing_about_its_store() -> &'static str {
        r#"{
            "version": "0.1.0", "started_at": "2026-09-09T08:00:00.000Z",
            "interval_seconds": 30, "collectors": [], "reporters": [],
            "findings": {"retained": 0, "capacity": 500, "total": 0, "dropped": 0}
        }"#
    }

    #[test]
    fn an_answer_from_an_agent_that_says_nothing_about_its_store_still_parses() {
        let status: AgentStatus =
            serde_json::from_str(an_agent_that_says_nothing_about_its_store()).expect("reads");

        assert!(
            status.store.is_none(),
            "an agent that does not report a store is not an agent with an empty one"
        );
    }

    #[test]
    fn a_field_this_console_does_not_know_does_not_make_the_answer_unreadable() {
        let from_a_newer_agent = r#"{
            "version": "0.2.0", "started_at": "2026-09-09T08:00:00.000Z",
            "interval_seconds": 30, "collectors": [], "reporters": [],
            "findings": {"retained": 0, "capacity": 500, "total": 0, "dropped": 0},
            "store": {"records": {"held": 4, "ceiling": 10}, "something_new": 7},
            "another_thing_entirely": {"nested": true}
        }"#;

        let status: AgentStatus = serde_json::from_str(from_a_newer_agent).expect("reads");

        assert_eq!(
            status.store.expect("a store").records.held,
            4,
            "a field the console has never heard of must not cost it the fields it knows"
        );
    }

    #[test]
    fn an_agent_that_reports_no_store_puts_no_empty_store_on_the_wire() {
        let status: AgentStatus =
            serde_json::from_str(an_agent_that_says_nothing_about_its_store()).expect("reads");

        let wire = serde_json::to_value(&status).expect("serialises");

        assert!(wire.get("store").is_none(), "{wire}");
    }
}
