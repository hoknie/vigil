use serde::{Deserialize, Serialize};

use crate::{
    AgentBudget, BufferStatus, CollectorStatus, FindingsSummary, ReporterStatus, Rfc3339,
    StoreStatus,
};

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buffers: Option<Vec<BufferStatus>>,
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

    fn watching() -> AgentStatus {
        AgentStatus {
            version: "0.1.0".into(),
            started_at: "2026-09-09T08:00:00.000Z".into(),
            interval_seconds: 30,
            collectors: Vec::new(),
            reporters: Vec::new(),
            findings: crate::FindingsSummary::default(),
            silence: Silence::default(),
            budget: AgentBudget::default(),
            store: None,
            buffers: Some(vec![crate::BufferStatus {
                receiver: "ndjson".into(),
                pending: 12,
                pending_ceiling: 500,
                bytes: 8_792,
                bytes_ceiling: 4 * 1024 * 1024,
                dropped_total: 41,
                oldest_at: Some("2026-09-09T08:41:02.000Z".into()),
            }]),
            limitations: Vec::new(),
        }
    }

    #[test]
    fn an_agent_that_says_nothing_about_its_buffers_is_not_an_agent_with_none() {
        let older: AgentStatus =
            serde_json::from_str(an_agent_that_says_nothing_about_its_store()).expect("reads");

        assert!(
            older.buffers.is_none(),
            "a daemon built before the field is not a daemon with no receivers, and a console \
             that reads it as none would say every delivery is going out"
        );

        let none_configured = AgentStatus {
            buffers: Some(Vec::new()),
            ..watching()
        };
        let wire = serde_json::to_value(&none_configured).expect("serialises");
        assert_eq!(
            wire["buffers"],
            serde_json::json!([]),
            "and an agent with no receivers at all says so, which is a normal way to run"
        );
    }

    #[test]
    fn a_console_built_before_the_buffers_field_reads_an_answer_that_carries_it() {
        #[derive(Deserialize)]
        struct AsAnOlderConsoleSawIt {
            version: String,
            interval_seconds: u32,
            findings: crate::FindingsSummary,
        }

        let line = serde_json::to_string(&watching()).expect("serialises");
        let older: AsAnOlderConsoleSawIt =
            serde_json::from_str(&line).expect("a field it never heard of is not a broken answer");

        assert_eq!(older.version, "0.1.0");
        assert_eq!(older.interval_seconds, 30);
        assert_eq!(older.findings.capacity, 0);
    }

    #[test]
    fn an_agent_that_reports_no_store_puts_no_empty_store_on_the_wire() {
        let status: AgentStatus =
            serde_json::from_str(an_agent_that_says_nothing_about_its_store()).expect("reads");

        let wire = serde_json::to_value(&status).expect("serialises");

        assert!(wire.get("store").is_none(), "{wire}");
    }
}
