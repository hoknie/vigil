use serde::{Deserialize, Serialize};

use crate::{Counted, Rfc3339};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StoreStatus {
    #[serde(default)]
    pub records: Counted,
    #[serde(default)]
    pub bytes: Counted,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_at: Option<Rfc3339>,
    #[serde(default)]
    pub dropped: StoreDropped,
    #[serde(default)]
    pub damaged: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub journal_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoreDropped {
    #[serde(default)]
    pub at_the_ceiling: u64,
    #[serde(default)]
    pub past_the_window: u64,
}

impl StoreStatus {
    pub fn is_empty(&self) -> bool {
        self.records.held == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_store_that_holds_nothing_is_not_a_store_that_was_never_asked() {
        let empty = StoreStatus {
            records: Counted {
                held: 0,
                ceiling: 10_000,
            },
            ..StoreStatus::default()
        };

        assert!(empty.is_empty());
        assert_eq!(empty.oldest_at, None);
    }

    #[test]
    fn the_two_ceilings_travel_as_two_numbers_because_they_are_two_decisions() {
        let wire = serde_json::to_value(StoreStatus {
            dropped: StoreDropped {
                at_the_ceiling: 3,
                past_the_window: 41,
            },
            ..StoreStatus::default()
        })
        .expect("serialises");

        assert_eq!(wire["dropped"]["at_the_ceiling"], 3);
        assert_eq!(wire["dropped"]["past_the_window"], 41);
    }

    #[test]
    fn a_store_status_from_an_agent_that_reports_less_than_this_one_knows_still_reads() {
        let thin: StoreStatus = serde_json::from_str(r#"{"records": {"held": 4}}"#).expect("reads");

        assert_eq!(thin.records.held, 4);
        assert_eq!(thin.damaged, 0);
        assert_eq!(thin.journal_path, None);
    }
}
