use vigil_model::Severity;

use super::App;

use crate::ui::screens::findings;
use crate::ui::{Choosing, Column, Level, Screen, Sorting, holding};

const NOTHING_SORTS: &str =
    "Nothing on this screen sorts: it is one page, not a list of rows to put in an order.";

const GROUPS_ARE_THE_ORDER: &str =
    "The grouped view is an order already: press \u{2190} for the sockets view, which sorts.";

const NOTHING_FILTERS: &str = "Nothing to filter here: the severity floor and the column search \
                               live on the findings. Press / to search this list.";

impl App {
    pub(super) fn sortable(&self) -> Vec<&'static str> {
        match self.nav.at() {
            Screen::FINDINGS => findings::SORTED_BY.to_vec(),
            screen if holding(screen.name()).is_some() => self.pane_sortable(),
            _ => Vec::new(),
        }
    }

    pub(super) fn sorted(&self) -> Sorting {
        self.sorting
            .get(&self.nav.at())
            .copied()
            .unwrap_or_default()
    }

    pub(super) fn sorting(&mut self) {
        if holding(self.nav.at().name()).is_some() && self.pane_sortable().is_empty() {
            self.message = Some(GROUPS_ARE_THE_ORDER.to_string());
            return;
        }
        let columns = self.sortable();
        if columns.is_empty() {
            self.message = Some(NOTHING_SORTS.to_string());
            return;
        }
        let at = self.sorted().chosen();
        self.chooser
            .open(Choosing::Sort, Sorting::offered(&columns), at);
        self.level = Level::List;
    }

    pub(super) fn narrowing(&mut self) {
        if self.nav.at() != Screen::FINDINGS {
            self.message = Some(NOTHING_FILTERS.to_string());
            return;
        }
        let at = offered()
            .iter()
            .position(|option| option == &self.filtered())
            .unwrap_or(0);
        self.chooser.open(Choosing::Filter, offered(), at);
        self.level = Level::List;
    }

    pub(super) fn filtered(&self) -> String {
        match self.filter.column() {
            Column::Any => floor_named(self.filter.floor()),
            column => looking_in(column),
        }
    }

    pub(super) fn chose(&mut self) {
        let Some(what) = self.chooser.choosing() else {
            return;
        };
        let at = self.chooser.at();
        self.chooser.close();

        match what {
            Choosing::Sort => {
                let screen = self.nav.at();
                self.sorting.insert(screen, Sorting::of(at));
                self.list_cursor_to_the_top();
            }
            Choosing::Filter => {
                match at < Severity::KNOWN.len() {
                    true => {
                        self.filter.set_floor(Severity::KNOWN[at].clone());
                        self.filter.look_in(Column::Any);
                    }
                    false => {
                        let column = Column::ALL
                            .get(at - Severity::KNOWN.len())
                            .copied()
                            .unwrap_or_default();
                        self.filter.look_in(column);
                        self.filter.search_mut().start();
                    }
                }
                self.list_cursor_to_the_top();
            }
        }
        self.refresh_wanted = true;
    }

    fn list_cursor_to_the_top(&mut self) {
        self.nav.findings = crate::ui::Cursor::default();
        if let Some(panes) = self.panes_mut() {
            *panes.cursor_mut() = crate::ui::Cursor::default();
        }
    }
}

pub(super) fn offered() -> Vec<String> {
    let mut said: Vec<String> = Severity::KNOWN.iter().map(floor_named).collect();
    said.extend(Column::ALL.iter().map(|column| looking_in(*column)));
    said
}

fn floor_named(severity: &Severity) -> String {
    match severity {
        Severity::Info => "every severity".to_string(),
        other => format!("{} and above", other.as_str()),
    }
}

fn looking_in(column: Column) -> String {
    format!("search in {}", column.name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_severity_and_every_column_of_the_table_is_on_the_list_of_what_can_be_narrowed() {
        let offered = offered();

        assert_eq!(offered.len(), Severity::KNOWN.len() + Column::ALL.len());
        assert_eq!(offered[0], "every severity");
        for severity in Severity::KNOWN.iter().skip(1) {
            assert!(
                offered
                    .iter()
                    .any(|option| option.contains(severity.as_str())),
                "{} is not offered",
                severity.as_str()
            );
        }
        for column in Column::ALL {
            assert!(
                offered.iter().any(|option| option.contains(column.name())),
                "{} is not offered",
                column.name()
            );
        }
    }

    #[test]
    fn the_severity_floor_is_one_of_the_filters_and_not_a_key_of_its_own_any_more() {
        assert!(
            offered()
                .iter()
                .any(|option| option.contains("critical and above")),
            "the floor moved out of s and into f, and losing it on the way would be losing a \
             way of reading the findings"
        );
    }
}
