use std::collections::BTreeMap;

use super::narrowing::Narrowing;
use super::offered::{EVERY_KIND, floor_named, kind_chosen, looking_in, offered, showing_only};
use crate::ui::app::App;
use crate::ui::{Choosing, Column, Level, Reading, Screen, holding};

const NOTHING_FILTERS: &str = "Nothing to filter on this screen: it is one page, not a list of \
                               rows to narrow.";

impl App {
    pub(in crate::ui::app) fn narrowing(&mut self) {
        match self.nav.at() {
            Screen::FINDINGS if self.on_the_silenced() => {
                self.message = Some(crate::ui::app::silences::IN_THE_ORDER_OF_THE_FILES.to_string())
            }
            Screen::FINDINGS => {
                let offered = offered(&self.kinds_held());
                let at = match self.filter.kind() {
                    Some(kind) => offered
                        .iter()
                        .position(|option| kind_chosen(option).flatten().as_deref() == Some(kind)),
                    None => offered.iter().position(|option| option == &self.filtered()),
                }
                .unwrap_or(0);
                self.chooser.open(Choosing::Filter, offered, at);
                self.level = Level::List;
            }
            screen if holding(screen.name()).is_some() => self.narrowing_a_list(),
            _ => self.message = Some(NOTHING_FILTERS.to_string()),
        }
    }

    fn narrowing_a_list(&mut self) {
        let kinds = self.pane_kinds();
        if kinds.is_empty() {
            let facets = self.facets_offered();
            if facets.is_empty() {
                self.message = Some(
                    "This list has one kind of row in it and nothing to narrow it to. Press / \
                     to search it."
                        .to_string(),
                );
                return;
            }
            let offered = facets.iter().map(Narrowing::said).collect();
            self.chooser.open(Choosing::Filter, offered, 0);
            self.level = Level::List;
            return;
        }
        let at = kinds
            .iter()
            .position(|option| option == &self.narrowed_to())
            .unwrap_or(0);
        self.chooser.open(Choosing::Filter, kinds, at);
        self.level = Level::List;
    }

    pub(in crate::ui::app) fn facets_offered(&self) -> Vec<Narrowing> {
        let (Some(pane), Some(panes)) = (self.pane(), self.panes()) else {
            return Vec::new();
        };
        let Reading::Taken(snapshot) = self.view.reading(pane.reads()) else {
            return Vec::new();
        };
        let under = self
            .pane_row_under_the_cursor()
            .map(|row| pane.facets(snapshot, &row))
            .unwrap_or_default();
        let chosen = panes.only();
        if under.is_empty() && chosen.is_empty() {
            return Vec::new();
        }

        let mut offered = vec![Narrowing::Everything];
        offered.extend(under.into_iter().map(Narrowing::Only));
        offered.extend(chosen.iter().map(|facet| Narrowing::Any(facet.name)));
        offered
    }

    pub(super) fn narrowed_to_a_facet(&mut self, at: usize) {
        let Some(chosen) = self.facets_offered().into_iter().nth(at) else {
            return;
        };
        let Some(panes) = self.panes_mut() else {
            return;
        };
        match chosen {
            Narrowing::Everything => panes.narrow_to_nothing(),
            Narrowing::Only(facet) => panes.narrow_to(facet),
            Narrowing::Any(name) => panes.widen_facet(name),
        }
    }

    pub(in crate::ui::app) fn pane_kinds(&self) -> Vec<String> {
        let Some(pane) = self.pane() else {
            return Vec::new();
        };
        let toggles = pane.toggles();
        if toggles.is_empty() {
            return Vec::new();
        }

        let mut offered = vec![EVERY_KIND.to_string()];
        offered.extend(toggles.iter().map(|toggle| showing_only(toggle.name)));
        offered
    }

    pub(in crate::ui::app) fn narrowed_to(&self) -> String {
        let Some(panes) = self.panes() else {
            return EVERY_KIND.to_string();
        };
        let Some(pane) = self.pane() else {
            return EVERY_KIND.to_string();
        };
        let shown: Vec<&str> = pane
            .toggles()
            .into_iter()
            .map(|toggle| toggle.name)
            .filter(|name| !panes.hidden().iter().any(|hidden| hidden == name))
            .collect();

        match shown.as_slice() {
            [only] => showing_only(only),
            _ => EVERY_KIND.to_string(),
        }
    }

    pub(super) fn narrowed_a_list_to(&mut self, at: usize) {
        let Some(pane) = self.pane() else {
            return;
        };
        let toggles: Vec<&'static str> = pane.toggles().into_iter().map(|one| one.name).collect();
        let hidden: Vec<String> = match at.checked_sub(1).and_then(|at| toggles.get(at)) {
            None => Vec::new(),
            Some(only) => toggles
                .iter()
                .filter(|name| *name != only)
                .map(|name| (*name).to_string())
                .collect(),
        };

        if let Some(panes) = self.panes_mut() {
            panes.show_every_kind();
            for name in hidden {
                panes.toggle(&name);
            }
        }
    }

    pub(in crate::ui::app) fn kinds_held(&self) -> Vec<(String, usize)> {
        let mut held: BTreeMap<String, usize> = BTreeMap::new();
        for finding in &self.view.found.findings {
            *held.entry(finding.kind.as_str().to_string()).or_default() += 1;
        }
        held.into_iter().collect()
    }

    pub(in crate::ui::app) fn filtered(&self) -> String {
        match self.filter.column() {
            Column::Any => floor_named(self.filter.floor()),
            column => looking_in(column),
        }
    }
}
