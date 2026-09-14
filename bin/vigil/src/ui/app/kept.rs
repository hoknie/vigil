use std::rc::Rc;

use vigil_model::Snapshot;
use vigil_view::{Index, Pane, RowKey, Sorting, listed};

use super::App;

use crate::ui::Reading;
use crate::ui::screens::pane;

impl App {
    pub(super) fn pane_rows(&self) -> Rc<Vec<RowKey>> {
        let (Some(showing), Some(pane)) = (self.showing_pane(), self.pane()) else {
            return Rc::default();
        };
        let hidden = showing.hidden();
        let opened = showing.opened();
        let asked = pane::asked(&showing, &hidden, &opened);
        let question = self.rows_question(&showing, &asked);

        self.rows_seen.get_or(question, || {
            let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
                return Rc::new(pane::rows(&self.view, pane.as_ref(), &asked));
            };
            let bare = vigil_view::Showing {
                search: "",
                sorting: Sorting::default(),
                ..asked
            };
            let indexed = self.rows_question(&showing, &bare);
            match self.pane_index(indexed.clone(), pane.as_ref(), snapshot, &bare) {
                Some(index) => {
                    Rc::new(self.listed_from(indexed, pane.as_ref(), snapshot, &asked, &index))
                }
                None => Rc::new(pane::rows(&self.view, pane.as_ref(), &asked)),
            }
        })
    }

    fn pane_index(
        &self,
        indexed: super::RowsAsked,
        pane: &dyn Pane,
        snapshot: &Snapshot,
        bare: &vigil_view::Showing<'_>,
    ) -> Option<Rc<Index>> {
        self.index_seen
            .get_or(indexed, || pane.index(snapshot, bare).map(Rc::new))
    }

    fn listed_from(
        &self,
        indexed: super::RowsAsked,
        pane: &dyn Pane,
        snapshot: &Snapshot,
        asked: &vigil_view::Showing<'_>,
        index: &Index,
    ) -> Vec<RowKey> {
        let search = asked.search.to_lowercase();
        let within: Option<Rc<Vec<usize>>> = match self.found_seen.borrow().as_ref() {
            Some((same, before, found))
                if *same == indexed && !before.is_empty() && search.contains(before.as_str()) =>
            {
                Some(Rc::clone(found))
            }
            _ => None,
        };

        let (found, rows) = listed(
            pane,
            snapshot,
            asked,
            index,
            within.as_deref().map(Vec::as_slice),
        );
        *self.found_seen.borrow_mut() = Some((indexed, search, Rc::new(found)));
        rows
    }

    pub(super) fn pane_tally(&self) -> String {
        let (Some(showing), Some(pane)) = (self.showing_pane(), self.pane()) else {
            return String::new();
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return String::new();
        };
        let hidden = showing.hidden();
        let opened = showing.opened();
        let asked = pane::asked(&showing, &hidden, &opened);
        let question = self.rows_question(&showing, &asked);
        let rows = self.pane_rows();

        self.tally_seen.get_or(question, || {
            let bare = vigil_view::Showing {
                search: "",
                sorting: Sorting::default(),
                ..asked
            };
            let counted = self.rows_question(&showing, &bare);
            match self
                .counts_seen
                .get_or(counted, || pane.counts(snapshot, &bare).map(Rc::new))
            {
                Some(counts) => pane.tally_listed(snapshot, &asked, &rows, &counts),
                None => pane.tally(snapshot, &asked, rows.len()),
            }
        })
    }

    fn rows_question(
        &self,
        showing: &pane::Showing<'_>,
        asked: &vigil_view::Showing<'_>,
    ) -> super::RowsAsked {
        (
            self.view.readings.generation(),
            self.nav.at(),
            showing.at,
            format!("{asked:?}"),
        )
    }
}
