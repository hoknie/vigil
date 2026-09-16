use std::rc::Rc;

use vigil_model::Snapshot;
use vigil_view::{Assembled, Index, Pane, Sorting, listing};

use super::App;

use crate::ui::Reading;
use crate::ui::screens::pane;
use crate::ui::types::cache::{Listed, Shown};

impl App {
    pub(super) fn pane_listed(&self) -> Rc<Listed> {
        let Some(showing) = self.showing_pane() else {
            return Rc::new(Listed::of(Rc::new(Shown::built(Vec::new()))));
        };
        let hidden = showing.hidden();
        let opened = showing.opened();
        let asked = pane::asked(&showing, &hidden, &opened);
        let question = self.rows_question(&showing, &asked);

        self.listed_seen
            .get_or(question, || Rc::new(Listed::of(self.pane_rows())))
    }

    pub(super) fn pane_rows(&self) -> Rc<Shown> {
        let (Some(showing), Some(pane)) = (self.showing_pane(), self.pane()) else {
            return Rc::new(Shown::built(Vec::new()));
        };
        let hidden = showing.hidden();
        let opened = showing.opened();
        let asked = pane::asked(&showing, &hidden, &opened);
        let question = self.rows_question(&showing, &asked);

        self.rows_seen.get_or(question, || {
            let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
                return Rc::new(Shown::built(pane::rows(&self.view, pane.as_ref(), &asked)));
            };
            let narrowed = vigil_view::Showing {
                search: "",
                sorting: Sorting::default(),
                ..asked
            };
            let bare = vigil_view::Showing {
                only: &[],
                ..narrowed
            };
            let indexed = self.rows_question(&showing, &bare);
            match self.pane_index(indexed, pane.as_ref(), snapshot, &bare) {
                Some(index) => {
                    let assembled = self.listed_from(
                        self.rows_question(&showing, &narrowed),
                        pane.as_ref(),
                        snapshot,
                        &asked,
                        &index,
                    );
                    Rc::new(Shown::indexed(index, assembled))
                }
                None => Rc::new(Shown::built(pane::rows(&self.view, pane.as_ref(), &asked))),
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
        narrowed: super::RowsAsked,
        pane: &dyn Pane,
        snapshot: &Snapshot,
        asked: &vigil_view::Showing<'_>,
        index: &Index,
    ) -> Assembled {
        let search = asked.search.to_lowercase();
        let within: Option<Rc<Vec<usize>>> = match self.found_seen.borrow().as_ref() {
            Some((same, before, found))
                if *same == narrowed && !before.is_empty() && search.contains(before.as_str()) =>
            {
                Some(Rc::clone(found))
            }
            _ => None,
        };

        let (found, assembled) = listing(
            pane,
            snapshot,
            asked,
            index,
            within.as_deref().map(Vec::as_slice),
        );
        *self.found_seen.borrow_mut() = Some((narrowed, search, Rc::new(found)));
        assembled
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
        let shown = self.pane_rows();

        self.tally_seen.get_or(question, || {
            let bare = vigil_view::Showing {
                search: "",
                sorting: Sorting::default(),
                only: &[],
                ..asked
            };
            let counted = self.rows_question(&showing, &bare);
            match self
                .counts_seen
                .get_or(counted, || pane.counts(snapshot, &bare).map(Rc::new))
            {
                Some(counts) => pane.tally_listed(snapshot, &asked, &shown.rows(), &counts),
                None => pane.tally(snapshot, &asked, shown.rows().len()),
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
