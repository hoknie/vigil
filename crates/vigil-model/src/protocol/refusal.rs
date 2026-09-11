use serde::{Deserialize, Serialize};

use crate::CollectorState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorRefusal {
    pub state: CollectorState,
    pub reason: String,
}

impl CollectorRefusal {
    pub fn new(state: CollectorState, reason: impl Into<String>) -> Self {
        CollectorRefusal {
            state,
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_carries_the_state_the_summary_names_and_the_words_the_daemon_used() {
        let wire = serde_json::to_value(CollectorRefusal::new(
            CollectorState::Unavailable,
            "auditd is not running",
        ))
        .expect("serialises");

        assert_eq!(wire["state"], "unavailable");
        assert_eq!(wire["reason"], "auditd is not running");
    }

    #[test]
    fn a_refusal_in_a_state_this_console_has_not_heard_of_is_shown_rather_than_dropped() {
        let from_a_newer_daemon: CollectorRefusal =
            serde_json::from_str(r#"{"state": "throttled", "reason": "too many readings"}"#)
                .expect("reads");

        assert_eq!(from_a_newer_daemon.state.as_str(), "throttled");
        assert!(from_a_newer_daemon.state.is_trouble());
    }
}
