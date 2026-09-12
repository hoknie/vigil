use vigil_model::{Finding, Severity};

use crate::ui::{Column, Search};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Filter {
    floor: Severity,
    search: Search,
    column: Column,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            floor: Severity::Info,
            search: Search::default(),
            column: Column::default(),
        }
    }
}

impl Filter {
    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn search_mut(&mut self) -> &mut Search {
        &mut self.search
    }

    pub fn holding_back(&self) -> bool {
        rank(&self.floor) > 0 || self.search.holding_back()
    }

    pub fn column(&self) -> Column {
        self.column
    }

    pub fn look_in(&mut self, column: Column) {
        self.column = column;
    }

    pub fn floor(&self) -> &Severity {
        &self.floor
    }

    pub fn set_floor(&mut self, floor: Severity) {
        self.floor = floor;
    }

    pub fn passing<'a>(&self, findings: &'a [Finding]) -> Vec<&'a Finding> {
        findings
            .iter()
            .filter(|finding| {
                self.above_the_floor(finding) && self.search.matches(&self.column.haystack(finding))
            })
            .collect()
    }

    fn above_the_floor(&self, finding: &Finding) -> bool {
        match &finding.severity {
            Severity::Unknown(_) => true,
            severity => rank(severity) >= rank(&self.floor),
        }
    }

    pub fn clear(&mut self) {
        *self = Filter::default();
    }

    pub fn describe(&self) -> String {
        let matching = match self.column {
            Column::Any => format!("matching {:?}", self.search.query()),
            column => format!("{} matching {:?}", column.name(), self.search.query()),
        };
        match (rank(&self.floor), self.search.holding_back()) {
            (0, false) => "everything the agent still holds".to_string(),
            (0, true) => matching,
            (_, false) => format!("{} and above", self.floor.as_str()),
            (_, true) => format!("{} and above, {matching}", self.floor.as_str()),
        }
    }
}

fn rank(severity: &Severity) -> u8 {
    match severity {
        Severity::Info => 0,
        Severity::Low => 1,
        Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
        Severity::Unknown(_) => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::fixture;

    fn searching_for(wanted: &str) -> Filter {
        let mut filter = Filter::default();
        filter.search_mut().start();
        for character in wanted.chars() {
            filter.search_mut().type_character(character);
        }
        filter.search_mut().accept();
        filter
    }

    fn findings() -> Vec<Finding> {
        let mut listening =
            fixture::finding("A new listening port on 0.0.0.0:4444", Severity::Critical);
        listening.finding_key = "port.listen|tcp|0.0.0.0:4444".into();
        let mut login = fixture::finding("A user logged in from a new address", Severity::Low);
        login.finding_key = "user|session|deploy|pts/0".into();
        login.kind = vigil_model::Kind::from("user.session.new".to_string());
        vec![listening, login]
    }

    #[test]
    fn a_fresh_console_holds_nothing_back() {
        let filter = Filter::default();

        assert!(!filter.holding_back());
        assert_eq!(filter.passing(&findings()).len(), 2);
        assert!(filter.describe().contains("everything"));
    }

    #[test]
    fn the_floor_hides_the_quiet_ones_and_names_itself() {
        let mut filter = Filter::default();
        filter.set_floor(Severity::Critical);

        let all = findings();
        let passing = filter.passing(&all);

        assert_eq!(passing.len(), 1);
        assert_eq!(passing[0].severity, Severity::Critical);
        assert!(
            filter.describe().contains("critical and above"),
            "{}",
            filter.describe()
        );
    }

    #[test]
    fn the_floor_a_reader_put_back_to_the_bottom_holds_nothing_back_again() {
        let mut filter = Filter::default();
        filter.set_floor(Severity::Critical);

        filter.set_floor(Severity::Info);

        assert!(!filter.holding_back());
        assert_eq!(filter.passing(&findings()).len(), 2);
    }

    #[test]
    fn a_severity_this_build_does_not_know_is_never_held_back_by_the_floor() {
        let mut strange = fixture::finding("something from a newer agent", Severity::Info);
        strange.severity = Severity::Unknown("catastrophic".into());
        let mut filter = Filter::default();
        filter.set_floor(Severity::Critical);

        let all = [strange];
        let passing = filter.passing(&all);

        assert_eq!(
            passing.len(),
            1,
            "a console must not decide that what it cannot rank does not matter"
        );
    }

    #[test]
    fn a_search_reaches_the_title_the_kind_and_the_object_key() {
        let all = findings();

        for wanted in ["LISTENING", "port.listen", "0.0.0.0:4444"] {
            let filter = searching_for(wanted);

            let passing = filter.passing(&all);
            assert_eq!(
                passing.len(),
                1,
                "{wanted} matched {} findings",
                passing.len()
            );
            assert!(passing[0].title.contains("listening port"), "{wanted}");
        }
    }

    #[test]
    fn a_search_narrowed_to_one_column_is_not_answered_by_another() {
        let all = findings();
        let mut in_the_title = searching_for("port.listen");
        in_the_title.look_in(Column::Title);

        assert!(
            in_the_title.passing(&all).is_empty(),
            "the object key carries those words and the title does not, so a search of the \
             title has nothing to find"
        );
        assert!(
            in_the_title.describe().contains("TITLE"),
            "{}",
            in_the_title.describe()
        );

        let mut in_the_object = searching_for("port.listen");
        in_the_object.look_in(Column::Object);
        assert_eq!(in_the_object.passing(&all).len(), 1);
    }

    #[test]
    fn the_floor_is_set_by_name_as_well_as_stepped_and_reads_back_the_same() {
        let mut filter = Filter::default();

        filter.set_floor(Severity::High);

        assert_eq!(filter.floor(), &Severity::High);
        assert_eq!(filter.passing(&findings()).len(), 1);
        assert!(filter.holding_back());
    }
}
