use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Rfc3339;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorStatus {
    pub name: String,
    pub state: CollectorState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run_at: Option<Rfc3339>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub items: usize,
    pub readings: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_run_at: Option<Rfc3339>,
    #[serde(default)]
    pub skipped: u64,
    pub failures: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    pub baseline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum CollectorState {
    Ok,
    Degraded,
    Unavailable,
    Off,
    Unknown(String),
}

impl CollectorState {
    pub fn as_str(&self) -> &str {
        match self {
            CollectorState::Ok => "ok",
            CollectorState::Degraded => "degraded",
            CollectorState::Unavailable => "unavailable",
            CollectorState::Off => "off",
            CollectorState::Unknown(other) => other,
        }
    }

    pub fn is_trouble(&self) -> bool {
        !matches!(self, CollectorState::Ok | CollectorState::Off)
    }
}

impl fmt::Display for CollectorState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<String> for CollectorState {
    fn from(text: String) -> Self {
        match text.as_str() {
            "ok" => CollectorState::Ok,
            "degraded" => CollectorState::Degraded,
            "unavailable" => CollectorState::Unavailable,
            "off" => CollectorState::Off,
            _ => CollectorState::Unknown(text),
        }
    }
}

impl From<CollectorState> for String {
    fn from(state: CollectorState) -> Self {
        state.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_this_console_has_not_heard_of_is_shown_rather_than_dropped() {
        let from_a_newer_daemon: CollectorState = "throttled".to_string().into();

        assert_eq!(
            from_a_newer_daemon,
            CollectorState::Unknown("throttled".into())
        );
        assert_eq!(from_a_newer_daemon.as_str(), "throttled");
    }

    #[test]
    fn the_known_states_round_trip_through_their_wire_form() {
        for state in [
            CollectorState::Ok,
            CollectorState::Degraded,
            CollectorState::Unavailable,
            CollectorState::Off,
        ] {
            let wire = state.as_str().to_string();
            assert_eq!(CollectorState::from(wire), state);
        }
    }

    #[test]
    fn a_console_one_version_older_still_reads_a_status_it_did_not_expect() {
        let from_a_daemon_that_has_no_schedule = r#"{
            "name": "network", "state": "ok", "items": 4, "readings": 9, "failures": 0,
            "baseline": true
        }"#;

        let status: CollectorStatus =
            serde_json::from_str(from_a_daemon_that_has_no_schedule).expect("reads");

        assert_eq!(status.every_seconds, None);
        assert_eq!(status.next_run_at, None);
        assert_eq!(status.skipped, 0);
    }

    #[test]
    fn a_collector_that_does_not_report_a_period_is_not_a_collector_with_a_period_of_zero() {
        let silent = CollectorStatus {
            name: "network".into(),
            state: CollectorState::Ok,
            reason: None,
            last_run_at: None,
            duration_ms: None,
            items: 0,
            readings: 0,
            failures: 0,
            every_seconds: None,
            next_run_at: None,
            skipped: 0,
            last_error: None,
            baseline: false,
        };

        let wire = serde_json::to_value(&silent).expect("serialises");

        assert!(wire.get("every_seconds").is_none());
        assert!(wire.get("next_run_at").is_none());
        assert_eq!(wire["skipped"], 0);
    }

    #[test]
    fn a_collector_switched_off_is_not_a_collector_in_trouble() {
        assert!(!CollectorState::Off.is_trouble());
        assert!(!CollectorState::Ok.is_trouble());

        assert!(CollectorState::Degraded.is_trouble());
        assert!(CollectorState::Unavailable.is_trouble());
        assert!(CollectorState::Unknown("throttled".into()).is_trouble());
    }
}
