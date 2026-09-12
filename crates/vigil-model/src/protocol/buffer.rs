use serde::{Deserialize, Serialize};

use crate::Rfc3339;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferStatus {
    pub receiver: String,
    #[serde(default)]
    pub pending: u64,
    #[serde(default)]
    pub pending_ceiling: u64,
    #[serde(default)]
    pub bytes: u64,
    #[serde(default)]
    pub bytes_ceiling: u64,
    #[serde(default)]
    pub dropped_total: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_at: Option<Rfc3339>,
}

impl BufferStatus {
    pub fn is_empty(&self) -> bool {
        self.pending == 0
    }

    pub fn at_the_ceiling(&self) -> bool {
        self.pending >= self.pending_ceiling || self.bytes >= self.bytes_ceiling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full() -> BufferStatus {
        BufferStatus {
            receiver: "host-findings".into(),
            pending: 500,
            pending_ceiling: 500,
            bytes: 369_664,
            bytes_ceiling: 4 * 1024 * 1024,
            dropped_total: 41,
            oldest_at: Some("2026-09-11T06:41:44.110Z".into()),
        }
    }

    #[test]
    fn a_buffer_holding_nothing_is_not_a_buffer_nobody_asked_about() {
        let empty = BufferStatus {
            receiver: "ndjson".into(),
            pending_ceiling: 500,
            bytes_ceiling: 4 * 1024 * 1024,
            ..BufferStatus::default()
        };

        assert!(empty.is_empty());
        assert!(!empty.at_the_ceiling());
        assert_eq!(
            empty.oldest_at, None,
            "an empty buffer names no oldest finding rather than the epoch"
        );
    }

    #[test]
    fn a_number_of_findings_travels_with_the_ceiling_it_is_measured_against() {
        let wire = serde_json::to_value(full()).expect("serialises");

        assert_eq!(wire["pending"], 500);
        assert_eq!(
            wire["pending_ceiling"], 500,
            "487 waiting means nothing to a reader who cannot see what fits"
        );
        assert!(full().at_the_ceiling());
    }

    #[test]
    fn two_receivers_of_the_same_kind_are_told_apart_by_name_here_as_they_are_on_disk() {
        let first = BufferStatus {
            receiver: "ndjson".into(),
            pending: 3,
            ..BufferStatus::default()
        };
        let second = BufferStatus {
            receiver: "ndjson-2".into(),
            pending: 0,
            ..BufferStatus::default()
        };

        assert_ne!(
            first.receiver, second.receiver,
            "one row for two receivers hides whichever of them is behind"
        );
    }

    #[test]
    fn a_row_from_an_agent_that_says_less_than_this_one_knows_still_reads() {
        let thin: BufferStatus =
            serde_json::from_str(r#"{"receiver": "syslog", "pending": 4}"#).expect("reads");

        assert_eq!(thin.pending, 4);
        assert_eq!(thin.dropped_total, 0);
        assert_eq!(thin.bytes_ceiling, 0);
    }
}
