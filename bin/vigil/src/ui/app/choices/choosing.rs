use vigil_model::Severity;

use crate::ui::app::App;
use crate::ui::{Choosing, Column, Sorting, holding};

impl App {
    pub(in crate::ui::app) fn chose(&mut self) {
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
            Choosing::Kill(target) => {
                self.chose_a_way_of_killing(target, at);
                return;
            }
            Choosing::Delete(_) => {
                self.chose_to_delete();
                return;
            }
            Choosing::Filter if holding(self.nav.at().name()).is_some() => {
                match self.pane_kinds().is_empty() {
                    true => self.narrowed_to_a_facet(at),
                    false => self.narrowed_a_list_to(at),
                }
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
