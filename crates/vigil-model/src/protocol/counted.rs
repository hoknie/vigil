use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counted {
    #[serde(default)]
    pub held: u64,
    #[serde(default)]
    pub ceiling: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_number_and_the_ceiling_it_is_under_travel_together_or_neither_can_be_read() {
        let wire = serde_json::to_value(Counted {
            held: 1_284,
            ceiling: 10_000,
        })
        .expect("serialises");

        assert_eq!(wire["held"], 1_284);
        assert_eq!(wire["ceiling"], 10_000);
    }

    #[test]
    fn an_agent_that_reports_only_one_half_of_it_still_reads() {
        let half: Counted = serde_json::from_str(r#"{"held": 7}"#).expect("reads");

        assert_eq!(half.held, 7);
        assert_eq!(half.ceiling, 0);
    }
}
