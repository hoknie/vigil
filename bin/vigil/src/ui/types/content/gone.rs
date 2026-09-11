use vigil_model::{Finding, Rfc3339};

pub struct Gone {
    pub key: String,
    pub title: String,
    pub kind: String,
    pub last_seen: Rfc3339,
}

impl Gone {
    pub fn of(finding: &Finding, key: impl Into<String>) -> Gone {
        Gone {
            key: key.into(),
            title: finding.title.clone(),
            kind: finding.kind.as_str().to_string(),
            last_seen: finding.observed_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    #[test]
    fn what_is_remembered_about_a_row_that_went_away_is_what_the_finding_said_about_it() {
        let finding = fixture::finding("The inet table filter was deleted", Severity::High);

        let gone = Gone::of(&finding, "fw-table|inet filter");

        assert_eq!(gone.key, "fw-table|inet filter");
        assert_eq!(gone.title, "The inet table filter was deleted");
        assert_eq!(
            gone.last_seen, finding.observed_at,
            "the moment the agent last had the row in front of it is the moment it judged it, \
             and the reading it is gone from cannot say when that was"
        );
    }
}
