use crate::ui::app::App;
use crate::ui::screens::findings;
use crate::ui::{Choosing, Level, Screen, Sorting};

const NOTHING_SORTS: &str =
    "Nothing on this screen sorts: it is one page, not a list of rows to put in an order.";

const GROUPS_ARE_THE_ORDER: &str =
    "The grouped view is an order already: press \u{2190} for the sockets view, which sorts.";

impl App {
    pub(in crate::ui::app) fn sortable(&self) -> Vec<&'static str> {
        match self.nav.at() {
            Screen::FINDINGS => findings::SORTED_BY.to_vec(),
            screen if screen.draws_a_reading() => self.pane_sortable(),
            _ => Vec::new(),
        }
    }

    pub(in crate::ui::app) fn sorted(&self) -> Sorting {
        self.sorting
            .get(&self.nav.at())
            .copied()
            .unwrap_or_default()
    }

    pub(in crate::ui::app) fn sorting(&mut self) {
        if self.nav.at().draws_a_reading() && self.pane_sortable().is_empty() {
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
}
