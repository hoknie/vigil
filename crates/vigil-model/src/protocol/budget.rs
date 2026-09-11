use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_percent: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resident_kb: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_platform_that_cannot_measure_its_own_memory_says_so_instead_of_reporting_zero() {
        let unmeasured = AgentBudget {
            duty_percent: Some(0.02),
            resident_kb: None,
        };

        let wire = serde_json::to_value(&unmeasured).expect("serialises");

        assert!(
            wire.get("resident_kb").is_none(),
            "a field left out is 'not measured'; 0 would be a measurement"
        );
        assert_eq!(wire["duty_percent"], 0.02);
    }

    #[test]
    fn a_status_from_a_daemon_that_reports_no_budget_at_all_still_reads() {
        let nothing: AgentBudget = serde_json::from_str("{}").expect("reads");

        assert_eq!(nothing.duty_percent, None);
        assert_eq!(nothing.resident_kb, None);
    }
}
