use vigil_model::Finding;
use vigil_view::time_of_day;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Column {
    #[default]
    Any,
    Time,
    Severity,
    Kind,
    Title,
    Object,
}

impl Column {
    pub const ALL: &'static [Column] = &[
        Column::Any,
        Column::Time,
        Column::Severity,
        Column::Kind,
        Column::Title,
        Column::Object,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Column::Any => "any value",
            Column::Time => "TIME",
            Column::Severity => "SEVERITY",
            Column::Kind => "KIND",
            Column::Title => "TITLE",
            Column::Object => "OBJECT",
        }
    }

    pub fn haystack(self, finding: &Finding) -> String {
        match self {
            Column::Any => format!(
                "{} {} {} {} {}",
                time_of_day(&finding.observed_at),
                finding.severity.as_str(),
                finding.kind.as_str(),
                finding.title,
                finding.finding_key
            ),
            Column::Time => time_of_day(&finding.observed_at).to_string(),
            Column::Severity => finding.severity.as_str().to_string(),
            Column::Kind => finding.kind.as_str().to_string(),
            Column::Title => finding.title.clone(),
            Column::Object => finding.finding_key.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use vigil_model::Severity;

    use super::*;
    use crate::ui::fixture;

    fn finding() -> Finding {
        let mut finding = fixture::finding("A new listening port on 0.0.0.0:4444", Severity::High);
        finding.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
        finding
    }

    #[test]
    fn a_search_over_one_column_reads_that_column_and_no_other() {
        let finding = finding();

        assert_eq!(Column::Title.haystack(&finding), finding.title);
        assert_eq!(Column::Object.haystack(&finding), finding.finding_key);
        assert!(
            !Column::Title.haystack(&finding).contains("port.listen|"),
            "a search in the title must not be answered by the object key"
        );
    }

    #[test]
    fn a_search_over_anything_reaches_every_column_the_table_draws() {
        let finding = finding();
        let whole = Column::Any.haystack(&finding);

        for column in Column::ALL.iter().skip(1) {
            for word in column.haystack(&finding).split_whitespace() {
                assert!(
                    whole.contains(word),
                    "{word} is in {} and not in the whole row",
                    column.name()
                );
            }
        }
    }

    #[test]
    fn every_column_the_table_draws_can_be_searched_in_by_its_own_heading() {
        for column in Column::ALL {
            assert!(!column.name().is_empty());
        }
        assert_eq!(Column::default(), Column::Any);
    }
}
